const SDK = require("@keepkey/keepkey-sdk")
const assert = require("assert")

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
        //handle no bridge

        console.log(config.apiKey)
        // console.log(sdk)
        // console.log(sdk.bitcoinGetAddress)

        //Unsigned TX
        let addressInfo = {
            addressNList: [0x80000000 + 49, 0x80000000 + 0, 0x80000000 + 0, 0, 0],
            // addressNList: [2147483732, 2147483648, 2147483648, 0, 0],
            coin: 'Bitcoin',
            scriptType: 'p2pkh',
            // scriptType: 'p2sh-p2wpkh',
            showDisplay: false
        }

        //push tx to api
        // console.log(kk.instance.SignTransaction())
        let timeStart = new Date().getTime()
        console.log(sdk.address)
        let response = await sdk.address.utxoGetAddress({
            address_n: addressInfo.addressNList,
            script_type:addressInfo.scriptType,
            coin:addressInfo.coin
        })
        console.log("response: ", response)
        let timeEnd = new Date().getTime()
        console.log("duration: ", (timeStart - timeEnd) / 1000)
        
        // Validate response
        assert(response, "No response received from device")
        assert(response.address, "No address in response")
        assert(typeof response.address === 'string', "Address is not a string")
        assert(response.address.length > 0, "Address is empty")
        
        // Validate Bitcoin legacy address format (starts with 1)
        if (!response.address.match(/^1[a-km-zA-HJ-NP-Z1-9]{25,34}$/)) {
            console.error("✗ Invalid Bitcoin legacy address format:", response.address)
            process.exit(1)
        }
        
        console.log("✓ Successfully got Bitcoin legacy address")

    } catch (e) {
        console.error("✗ Test failed:", e.message || e)
        process.exit(1)
    }
}

run_test()
