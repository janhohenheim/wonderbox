# Changelog

## 0.1.0

- Initial release

## 0.2.0

- Store factories as `Send + Sync`

## To be released

- Cleanup API
    - Remove `register_clone` and `register_default` from API
    - Rename `register_factory` to `register`
    - Rename `register_container` to `extend`
- Unify naming
    - Rename `#[resolve_dependencies]` to `#[autoresolvable]`
    - Rename `register_autoresolved` to `register_autoresolvable`
    - Rename `register!` to `register_autoresolvable!`
- Improve documentation
