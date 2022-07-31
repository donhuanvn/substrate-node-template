const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');

(async () => {
  const provider = new WsProvider('ws://127.0.0.1:9944')
  const api = await ApiPromise.create(provider)

  const keyring = new Keyring({ type: 'sr25519' })

  const alice = keyring.addFromUri('//Alice');
  const bob = keyring.addFromUri('//Bob');

  const txts = Array(100).fill(api.tx.balances.transfer(bob.address, 12345))

  await api.tx.utility.batch(txts).signAndSend(alice, ({ status }) => {
    if (status.isInBlock) {
      console.log(`included in ${status.asInBlock}`)
    }
  })
})()
