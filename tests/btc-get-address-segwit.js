const SDK = require("@keepkey/keepkey-sdk")


let spec = 'http://localhost:1646/spec/swagger.json'

let run_test = async function () {
    try {
        let config = {
            apiKey: process.env['SERVICE_KEY'] || '1fa0c776-eaa9-499d-a2e5-f76af6073912',
            pairingInfo:{
                name: process.env['SERVICE_NAME'] || 'KeepKey SDK Demo App',
                imageUrl: process.env['SERVICE_IMAGE_URL'] || 'https://github.com/BitHighlander/keepkey-desktop/raw/master/electron/icon.png',
                basePath:spec,
                url:"http://localhost:1646"
            }
        }
        //init
        const sdk = await SDK.KeepKeySdk.create(config)

        console.log(config.apiKey)

        // Native SegWit address (P2WPKH)
        let addressInfo = {
            addressNList: [0x80000000 + 84, 0x80000000 + 0, 0x80000000 + 0, 0, 0], // m/84'/0'/0'/0/0
            coin: 'Bitcoin',
            scriptType: 'p2wpkh',
            showDisplay: false
        }

        let timeStart = new Date().getTime()
        console.log("Getting native SegWit address...")
        let response = await sdk.address.utxoGetAddress({
            address_n: addressInfo.addressNList,
            script_type:addressInfo.scriptType,
            coin:addressInfo.coin,
            show_display: true,
        })
        console.log("response: ", response)
        let timeEnd = new Date().getTime()
        console.log("duration: ", (timeStart - timeEnd) / 1000)

        // Verify we got a bech32 address
        if (response.address && response.address.startsWith('bc1')) {
            console.log("✓ Successfully got native SegWit address")
        } else {
            console.error("✗ Expected native SegWit address (bc1...), got:", response.address)
            process.exit(1)
        }

    } catch (e) {
        console.error(e)
        process.exit(1)
    }
}

run_test() 