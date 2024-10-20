# Aptos (WIP)

The router folder was generated with `aptos move init --name Router`.

This module leverages Move 2.0 features and the `aptos` CLI requires the `--move-2` flag as recently as version `4.2.3`.

## Status

- [x] `register`
- [x] `sendMessage`
- [ ] `getMessageStatus`
- [ ] `recvMessage`
- [ ] `execMessage`
- [ ] `attestMessage`
- [ ] `pickUpMessage`
- [x] `updateAdmin`
- [x] `transferAdmin`
- [x] `claimAdmin`
- [x] `discardAdmin`
- [x] `addTransceiver`
- [x] `enableSendTransceiver`
- [x] `disableSendTransceiver`
- [x] `enableRecvTransceiver`
- [x] `disableRecvTransceiver`

- [ ] CI builds and enforces 100% coverage
- [ ] Example transceiver
- [ ] Example integrator

## Design

### Integrators

On-chain contracts which integrate with the Router will need to store a resource account capability in order to pass a resource account signer to the `register` and `send_message` functions of the Router. This allows the Router to assign a sequence tracker and integrator configuration directly to that signer, restrict use to that signer, and improve off-chain visibility. See [Resource Accounts](https://aptos.dev/en/build/smart-contracts/resource-accounts) for more info.

### Router

As of this writing, Aptos does not generally support dynamic dispatch (though they do support very specific dynamic dispatch through their [Fungible Asset](https://aptos.dev/en/build/smart-contracts/fungible-asset#dispatchable-fungible-asset-advanced)). This means that the Router cannot call arbitrary Transceivers on behalf of the integrator. It therefore must store intermediate message state and rely on a pull model for Transceivers. Similar to inbound attestations, the outbound messages will be stored in a Table on the Router.

This limitation means that the most effective way for an integrator to generically post a message and have the Transceivers pick it up in a single transaction will be to have a front-end / SDK use an [Aptos Script](https://aptos.dev/en/build/smart-contracts/scripts). In the future, it may be officially supported to use a [Dynamic Transaction Composer](https://github.com/aptos-foundation/AIPs/blob/main/aips/aip-102.md) to achieve this.

### Transceivers

Transceivers must be associated with exactly one Router instance and must expose a method to pick up a message from the Router by its `source_address` and `sequence`, passing in a signer which uniquely identifies the Transceiver (like its resource account signer). This signer must be the same known address used by Integrators when adding the Transceiver.

## Development

Style note: this code intentionally avoids the [dot (receiver) function call style](https://aptos.dev/en/build/smart-contracts/book/functions#dot-receiver-function-call-style) as it obscures the mutability of the reference used.

### Compile

```bash
aptos move compile --move-2 --named-addresses router=default
```

### Test

```bash
aptos move test --move-2 --named-addresses router=default
```

For coverage, add the `--coverage` flag.

```bash
aptos move test --move-2 --coverage --named-addresses router=default
```

And to view coverage or a module _after_ testing. e.g. for `router::integrator`

```bash
aptos move coverage source --module integrator --move-2 --named-addresses router=default
```
