# KVS

This is an implementation of a key-value store. It includes a CLI for querying the database and a library for use in applications.

Data is stored in a log-structured file store.

The implementation was created based on the walkthrough at [pingcap/talent-plan](https://github.com/pingcap/talent-plan).

## License

The test suite in `tests/tests.rs` is derived from the pingcap guide and licensed [CC-BY 4.0](https://opendefinition.org/licenses/cc-by/).

The CLI and lib.rs APIs are based on the descriptions in the pingcap guide but it is assumed that these APIs are not copyrightable.

All other code is original work and licensed according to the included license file.
