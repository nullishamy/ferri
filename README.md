# ferri

a fediverse backend written in rust.

## goals
- mastoapi compatible
- well suited for single user / selfhosted deployments
- feature-rich (reactions, quotes, ...)

## general structure
- AP/MastoAPI separation
- cli will handle admin / spinup
- main will handle business logic
- server will handle HTTP layer
