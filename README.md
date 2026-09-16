<!-- File internal version: v0.2 -->
# GoreeCloud Advanced Download Manager

**Product internal version:** 0.1.0  
**Release lifecycle:** Concept  
**Repository state:** Documentation and architecture foundation; no download engine or installable client is currently verified.

GoreeCloud Advanced Download Manager is the planned cross-platform download and transfer-orchestration application for Linux, Windows, and Android. Its target is a local-first common download engine with durable transfers, intelligent acceleration, queues, automation, privacy and security controls, browser integration, and authorized GoreeCloud ecosystem coordination.

## Current availability

There is currently **no supported build to install or run** from this repository. The repository contains product requirements, roadmap, governance, and architecture documentation. Do not interpret planned features as implemented functionality.

## Product direction

The implementation is intended to prioritize a durable local transfer core before adding multipart acceleration, browser interception, remote control, synchronization, or broader ecosystem integrations. BitTorrent and magnet transfers are planned to be delegated to GoreeCloud Swarm rather than duplicating its protocol stack.

## Documentation

- [Product specification](SPECIFICATIONS.md)
- [Feature roadmap](FEATURE-ROADMAP.md)
- [Current feature state](FEATURES.md)
- [Architecture](ARCHITECTURE.md)
- [User manual](USER-MANUAL.md)
- [Privacy policy](PRIVACY%20POLICY.md)
- [Security guidance](SECURITY.md)
- [Benefits](BENEFITS.md)
- [Competitive objectives](COMPETITIVE-OBJECTIVES.md)
- [Branding](BRANDING.md)
- [Repository notes](NOTES.md)
- [Changelog](CHANGELOG.md)

## Platform contract

`goreecloud.platform.yaml` records the current GoreeCloud Platform Contract declaration. All seven Integral Platform Systems are presently unaccepted for this application; metadata does not establish integration.

## Development

Implementation language, build system, packaging model, and client framework are not yet finalized. Those choices must be made against the Linux, Windows, Android, security, recovery, maintainability, and shared-engine requirements before source scaffolding is treated as the product foundation.
