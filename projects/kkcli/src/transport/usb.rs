use super::Transport;
use core::{cmp::min, iter::repeat, time::Duration};
use rusb::{ConfigDescriptor, Device, DeviceHandle, UsbContext};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct UsbTransport<T: UsbContext> {
    handle: Arc<Mutex<DeviceHandle<T>>>,
    in_endpoint_address: u8,
    out_endpoint_address: u8,
    in_packet_size: usize,
    out_packet_size: usize,
}

impl<T: UsbContext> UsbTransport<T> {
    /// Properly cleanup USB resources and reset device
    pub fn cleanup(&mut self) -> Result<(), rusb::Error> {
        tracing::info!("🧹 Cleaning up USB transport and resetting device...");
        
        // First try to reset the device communication
        if let Err(e) = self.reset() {
            tracing::warn!("⚠️ Transport reset failed during cleanup: {}", e);
        }
        
        // Try to reset the USB device itself
        match self.handle.lock().map_err(|_| rusb::Error::Other)?.reset() {
            Ok(()) => {
                tracing::info!("✅ USB device reset during cleanup successful");
            }
            Err(e) => {
                tracing::warn!("⚠️ USB device reset during cleanup failed: {}", e);
            }
        }
        
        // Note: Interface will be automatically released when DeviceHandle is dropped
        tracing::info!("✅ USB transport cleanup completed");
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    pub fn new(
        device: &Device<T>,
        interface_index: usize,
    ) -> Result<(Self, ConfigDescriptor, Arc<Mutex<DeviceHandle<T>>>), rusb::Error> {
        let config_descriptor = device.active_config_descriptor()?;
        let mut handle_arc = Arc::new(Mutex::new(device.open()?));

        let locked_handle = Arc::get_mut(&mut handle_arc)
            .ok_or(rusb::Error::Other)?
            .get_mut()
            .map_err(|_| rusb::Error::Other)?;
        
        locked_handle.reset()?;

        // Detaching the kernel driver is required to access HID devices on Mac OS and Linux.
        match locked_handle.set_auto_detach_kernel_driver(true) {
            Err(rusb::Error::NotSupported) => Ok(()),
            x => x,
        }?;

        Ok((
            Self::new_from_descriptor_and_handle(
                &config_descriptor,
                Arc::clone(&handle_arc),
                interface_index,
            )?,
            config_descriptor,
            handle_arc,
        ))
    }

    pub fn new_from_descriptor_and_handle(
        config_descriptor: &ConfigDescriptor,
        handle_arc_param: Arc<Mutex<DeviceHandle<T>>>,
        interface_index: usize,
    ) -> Result<Self, rusb::Error> {
        let locked_handle = handle_arc_param.lock().map_err(|_| rusb::Error::Other)?;

        let interface = config_descriptor
            .interfaces()
            .nth(interface_index)
            .ok_or(rusb::Error::NotFound)?;
        locked_handle.claim_interface(interface.number())?;

        let mut interface_descriptors = interface.descriptors();
        let interface_descriptor = interface_descriptors.next().ok_or(rusb::Error::NotFound)?;
        locked_handle.set_alternate_setting(interface.number(), 0)?;

        let mut endpoint_descriptors = interface_descriptor.endpoint_descriptors();

        let in_endpoint_descriptor = endpoint_descriptors.next().ok_or(rusb::Error::NotFound)?;
        assert_eq!(in_endpoint_descriptor.direction(), rusb::Direction::In);
        assert_eq!(
            in_endpoint_descriptor.transfer_type(),
            rusb::TransferType::Interrupt
        );

        let out_endpoint_descriptor = endpoint_descriptors.next().ok_or(rusb::Error::NotFound)?;
        assert_eq!(out_endpoint_descriptor.direction(), rusb::Direction::Out);
        assert_eq!(
            out_endpoint_descriptor.transfer_type(),
            rusb::TransferType::Interrupt
        );

        drop(locked_handle);

        Ok(Self {
            handle: handle_arc_param,
            in_endpoint_address: in_endpoint_descriptor.address(),
            out_endpoint_address: out_endpoint_descriptor.address(),
            in_packet_size: in_endpoint_descriptor.max_packet_size().into(),
            out_packet_size: out_endpoint_descriptor.max_packet_size().into(),
        })
    }

    fn read_packet(&self, buf: &mut Vec<u8>, timeout: Duration) -> Result<(), rusb::Error> {
        let mut packet = vec![0u8; self.in_packet_size];
        let len = self.handle.lock().map_err(|_| rusb::Error::Other)?.read_interrupt(
            self.in_endpoint_address,
            &mut packet,
            timeout,
        )?;
        assert_eq!(
            len,
            self.in_packet_size,
            "packet: {:?}",
            hex::encode(packet)
        );
        if !(len >= 1 && packet[0] == b'?') {
            return Err(rusb::Error::Other);
        }
        buf.extend_from_slice(&packet[1..]);
        Ok(())
    }

    /* // Commenting out this problematic method for now
    fn control_transfer(&self, req_type: u8, req: u8, val: u16, idx: u16, data: &[u8]) -> Result<Vec<u8>, Error> {
        let device_handle = self.device_handle.as_ref().ok_or(Error::DeviceNotConnected)?;
        let handle = device_handle.lock().unwrap();
        let timeout = std::time::Duration::from_secs(5);
        let mut buf = vec![0u8; 256]; // Standard buffer, might need adjustment
        // ... existing code ...
    }
    */
}

macro_rules! since {
    ($started:expr, $timeout:expr) => {
        $timeout
            .checked_sub($started.elapsed())
            .filter(|x| *x >= Duration::from_millis(1))
            .ok_or(rusb::Error::Timeout)
    };
}

impl<T: UsbContext> Transport for UsbTransport<T> {
    type Error = rusb::Error;
    fn write(&mut self, msg: &[u8], timeout: Duration) -> Result<usize, Self::Error> {
        let started = Instant::now();
        let mut packet = Vec::<u8>::with_capacity(self.out_packet_size);
        for chunk in msg.chunks(self.out_packet_size - 1) {
            packet.clear();
            packet.push(b'?');
            packet.extend_from_slice(chunk);
            packet.extend(repeat(0).take(self.out_packet_size - packet.len()));
            debug_assert_eq!(packet.len(), self.out_packet_size);

            let written_len = self.handle.lock().map_err(|_| rusb::Error::Other)?.write_interrupt(
                self.out_endpoint_address,
                &packet,
                since!(started, timeout)?,
            )?;
            assert_eq!(written_len, packet.len());
        }
        Ok(msg.len())
    }
    fn read(&mut self, buf: &mut Vec<u8>, timeout: Duration) -> Result<(), Self::Error> {
        let mut packet = Vec::<u8>::with_capacity(self.in_packet_size);
        let started = Instant::now();
        self.read_packet(&mut packet, timeout)?;

        if !(packet.len() >= 8 && packet[0] == b'#' && packet[1] == b'#') {
            return Err(rusb::Error::Other);
        }
        let msg_len: usize = u32::from_be_bytes(packet[4..8].try_into().map_err(|_| rusb::Error::Other)?)
            .try_into()
            .map_err(|_| rusb::Error::Other)?;

        let mut len_remaining = 8 + msg_len;
        loop {
            buf.extend_from_slice(&packet[..min(len_remaining, packet.len())]);
            len_remaining = len_remaining.saturating_sub(packet.len());

            if len_remaining == 0 {
                break;
            }

            packet.clear();
            self.read_packet(&mut packet, since!(started, timeout)?)?;
        }

        Ok(())
    }
    fn reset(&mut self) -> Result<(), Self::Error> {
        // First flush any pending read data
        const FLUSH_TIMEOUT: Duration = Duration::from_millis(10);
        let mut buf = vec![0u8; self.in_packet_size];
        loop {
            match self.handle.lock().map_err(|_| rusb::Error::Other)?.read_interrupt(
                self.in_endpoint_address,
                &mut buf,
                FLUSH_TIMEOUT,
            ) {
                Ok(0) => break,
                Ok(_) => (),
                Err(rusb::Error::Timeout) => break,
                Err(rusb::Error::Overflow) => (),
                Err(x) => return Err(x),
            }
            buf.fill(0);
        }
        
        // Then perform actual USB device reset
        match self.handle.lock().map_err(|_| rusb::Error::Other)?.reset() {
            Ok(()) => {
                tracing::info!("✅ USB device reset successful");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("⚠️ USB device reset failed: {}, continuing anyway", e);
                Ok(()) // Don't fail if reset doesn't work
            }
        }
    }
}
