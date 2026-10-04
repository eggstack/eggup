# Changelog

## Unreleased

This crate has never been published. There is no migration requirement for
downstream consumers, and no crates.io version of `eggup-curl` exists.

Fixes from the workspace bug audit; no API change.

- **Security: an `https` request could follow a redirect down to `http`.**
  `CurlConfig::allowed_protocols` was forwarded as both `--proto` and
  `--proto-redir`, and the default list contains `http`. A `302 Location:
  http://…` was therefore followed in cleartext with no error and no
  diagnostic, and the cleartext body was promoted to the destination as an
  ordinary success. `--proto-redir` is now narrowed per request so an `https`
  URL never lists `http` as a permitted redirect target, matching the strict
  downgrade denial the native `eggup-eggfetch` adapter enforces. A cleartext
  request may still be redirected to either scheme, and a configuration that
  allows only `http` still emits a well-formed list.
- **Fixed: a mislabelled timeout phase.** For curl exit 28 the connect phase was
  inferred from elapsed time against the effective connect ceiling. When the
  effective connect and total ceilings are equal — as they are under a
  one-second caller limit, which the adapter's minimums never shrink — that test
  held for every exit 28, so a total-deadline timeout was reported as a connect
  timeout. Coincident ceilings now report the weaker, always-true `total` phase.

- Acquisition M005 (unpublished): new `eggup-curl` external-curl adapter plus
  `AcquisitionError::Unavailable` and `eggup-acquisition::ComposedTransport`
  with `CompositionPolicy::{UnavailableOnly (default), UnavailableOrTransport}`.
  Curl-only binaries avoid an embedded HTTP/TLS stack; Eggfetch-only binaries
  avoid curl; dual binaries compose both for the same exact URL. Exact 404
  remains terminal `NotFound`; default fallback occurs only on unavailability.
  No `eggup-core` change; no Gregg modification or dependency. No publication
  or consumer migration performed.

## 0.1.2

Source version only — not published. The crate shares the workspace `0.1.2`
source version and has no crates.io release. See the `Unreleased` section above
for what the crate contains.
