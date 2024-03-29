# PeerMe DAO Smart Contracts

[![](https://img.shields.io/twitter/follow/PeerMeHQ?color=%23555555&label=Follow%20PeerMe&logo=twitter&style=for-the-badge)](https://twitter.com/PeerMeHQ)
[![](https://dcbadge.vercel.app/api/server/sDeejyk3VR)](https://discord.gg/sDeejyk3VR)

Smart contract used for managing and running DAOs through [PeerMe](https://peerme.io) on MultiversX Blockchain.

Specifically, two smart contracts:

- The Entity Template: Is the actual DAO smart contract that users interact with
- The Manager: Deploys and manages instances of the Entity Template smart contract & contains other utilities

Find the mentioned smart contracts on the Explorer:

- Entity Template: [erd1qqqqqqqqqqqqqpgqces4kdydtp9ea29pymjjyg7vcfqfllly27rsv3qats](https://explorer.elrond.com/accounts/erd1qqqqqqqqqqqqqpgqces4kdydtp9ea29pymjjyg7vcfqfllly27rsv3qats)
- Manager: [erd1qqqqqqqqqqqqqpgqtatmxjhlxkehl37u5kz9tz7sm450xd7f27rsppynzj](https://explorer.elrond.com/accounts/erd1qqqqqqqqqqqqqpgqtatmxjhlxkehl37u5kz9tz7sm450xd7f27rsppynzj)

## Documentation

You can find extensive Documentation about DAOs & their supporting tools in our [Knowledge Base](https://know.peerme.io):

- To learn more about DAOs in general, read our [Overview](https://know.peerme.io/daos/overview.html) page.
- To create your own DAO, follow the simple steps on the [Setup](https://know.peerme.io/daos/setup.html) page.
- To better understand roles & permissions, check out the [Roles & Permissions](https://know.peerme.io/daos/permissions.html) page.
- To learn about the technical concepts, visit the [Technical](https://know.peerme.io/daos/technical.html) page.

## Development

### Requirements

- The Manager smart contract must possess the `ESDTRoleLocalMint` role for the configured token of `boost-reward-token-id` – [SUPERPOWER-6f4cee](https://explorer.elrond.com/tokens/SUPERPOWER-6f4cee) in our case

### Deploy

Before deploying the smart contract to the blockchain, be sure to:

1. Remove the `exit` part within the `deploy` function in `interaction/manager.sh` to disable deploy protection.
2. Configure all variables within `mxpy.data-storage.json` for the corresponding network.
3. Connect & unlock your Ledger device with the Elrond app open, ready to sign the deploy transaction.

```bash
. ./interaction/manager.sh && deploy
```

### Upgrade

To upgrade the Manager smart contract:

```bash
. ./interaction/manager.sh && upgrade
```

To upgrade the Entity Template smart contract:

```bash
. ./interaction/manager.sh && upgradeEntityTemplate
```

### Testing

You can run the tests with:

```bash
cargo test
```

## Security Vulnerabilities

Please review [our security policy](../../security/policy) on how to report security vulnerabilities.

## Credits

- [Micha Vie](https://github.com/michavie)
- [All Contributors](../../contributors)

## License

The GNU GENERAL PUBLIC LICENSE v3.0. Please see [License File](LICENSE) for more information.
