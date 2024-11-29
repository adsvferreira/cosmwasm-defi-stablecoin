# Cosmwasm version of Cyfrin/foundry-defi-stablecoin-f23

This project is a CosmWasm version of the Founfry DeFi Stablecoin project developed by PatrickAlphaC as a section of the Cyfrin Foundry Solidity Course (https://github.com/Cyfrin/foundry-defi-stablecoin-f23).

<br>

## Neutron Mainnet Addresses

### DSC

Overcollaterized Stable Coin (CW20) minted by `DSC ENGINE`

```bash
neutron1pd26yx0t56rya0xxeww80fzt3ag59uje32zvuke835lt8s5hakxsechnsf
```

### ORACLE

Pyth Network Oracle's Client
<br>
https://docs.pyth.network/price-feeds/use-real-time-data/cosmwasm

```bash
neutron1w8gq74jdmwq307vctln6p9hzpcvk3ync5wyu6u32shr543npp8aspejwqy
```

### DSC ENGINE

The core of the Decentralized Stablecoin system.
<br>
Handles all the logic for minting and redeeming DSC, as well as depositing and withdrawing collateral.
<br>
This contract is based on the MakerDAO DSS system.

```bash
neutron1x2c2k0a278gpduf8c2svyg8n8xs3twvkzky3703vzh780d8hqdqq6zmvrn
```

<br>

## Running integration tests

```bash
cargo test
```

## Compiling contracts using WasmKit

To compile your contracts:

```bash
wasmkit compile
```

## Running `deploy_and_test` script using WasmKit

```bash
wasmkit run scripts/deploy_and_test.ts --network <network>
```
