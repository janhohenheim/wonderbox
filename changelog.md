# Changelog

## 0.1.0

- Initial release

## 0.2.0

- Store factories as `Send + Sync`

## 0.3.0

- Cleanup API
    - Remove `register_clone` and `register_default` from API
    - Rename `register_factory` to `register`
    - Rename `register_container` to `extend`
- Unify naming
    - Rename `#[resolve_dependencies]` to `#[autoresolvable]`
    - Rename `register_autoresolved` to `register_autoresolvable`
    - Rename `register!` to `register_autoresolvable!`
- Improve documentation
- Make `register_autoresolvable!` work with any simple wrapper around a trait object

## 0.4.0

- `resolve` now unwraps `T`, printing a helpful message if it fails. 
  The old behavior can still be accessed via `try_resolve`
- Add support for the registration of generics
