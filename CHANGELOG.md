## [0.1.5](https://github.com/andoriyu/uclicious/compare/v0.1.4...v0.1.5) (2020-04-20)


### Bug Fixes

* **sys:** Update to latest snapshot of libucl ([01c8451](https://github.com/andoriyu/uclicious/commit/01c8451))


### Features

* **derive:** Ability to add chunks ([79631ad](https://github.com/andoriyu/uclicious/commit/79631ad))
* **derive:** Ability to add pre_sync_hook ([958d82a](https://github.com/andoriyu/uclicious/commit/958d82a))
* **parser:** Introduce VaribleHandler trait ([6c7f4e0](https://github.com/andoriyu/uclicious/commit/6c7f4e0))
* **vh:** Add simple and safe iterface to parser ([0406473](https://github.com/andoriyu/uclicious/commit/0406473))
* **vh:** Make variables handlers into features ([139a9de](https://github.com/andoriyu/uclicious/commit/139a9de))
* **vh/compound:** Introduce Compound VarHandler ([73f7229](https://github.com/andoriyu/uclicious/commit/73f7229))
* **vh/env:** Introduce EnvVar variable handler ([a13f0fa](https://github.com/andoriyu/uclicious/commit/a13f0fa))



## [0.1.4](https://github.com/andoriyu/uclicious/compare/v0.1.3...v0.1.4) (2020-04-05)


### Bug Fixes

* **iter:** iterators for implicit arrays ([#12](https://github.com/andoriyu/uclicious/issues/12)) ([af2a5cc](https://github.com/andoriyu/uclicious/commit/af2a5cc))
* **priority:** Priority incorrectly capped at 15 ([#15](https://github.com/andoriyu/uclicious/issues/15)) ([5ce2514](https://github.com/andoriyu/uclicious/commit/5ce2514))


### Features

* **object:** Ability to clone and deepclone objects ([38de0fc](https://github.com/andoriyu/uclicious/commit/38de0fc))
* **object:** Implement Ord ([#14](https://github.com/andoriyu/uclicious/issues/14)) ([1fd4f73](https://github.com/andoriyu/uclicious/commit/1fd4f73))
* **object:** Implement PartialEq ([#10](https://github.com/andoriyu/uclicious/issues/10)) ([5b7b78b](https://github.com/andoriyu/uclicious/commit/5b7b78b))
* Derive mandatory traits for Priority ([9770705](https://github.com/andoriyu/uclicious/commit/9770705))



## [0.1.3](https://github.com/andoriyu/uclicious/compare/0.1.2...v0.1.3) (2020-04-04)


### Bug Fixes

* **derive:** automatically convert TryFrom errors ([c3273e5](https://github.com/andoriyu/uclicious/commit/c3273e5))


### Features

* **derive:** Map conversion helper ([51056a6](https://github.com/andoriyu/uclicious/commit/51056a6))



## [0.1.2](https://github.com/andoriyu/uclicious/compare/0.1.1...0.1.2) (2020-03-22)


### Features

* Ability to "wrap" other errors inside ObjectError. ([7e4db2c](https://github.com/andoriyu/uclicious/commit/7e4db2c))
* Add TryInto trait for simplicity of conversion. ([5f1cff2](https://github.com/andoriyu/uclicious/commit/5f1cff2))



## [0.1.1](https://github.com/andoriyu/uclicious/compare/40f6bd4...0.1.1) (2020-03-16)


### Features

* Ability to call set_filevars on a builder. ([7e8724f](https://github.com/andoriyu/uclicious/commit/7e8724f))
* Ability to register variables on a builder. ([a2f3ede](https://github.com/andoriyu/uclicious/commit/a2f3ede))
* parse time object into f64 and Duration ([40f6bd4](https://github.com/andoriyu/uclicious/commit/40f6bd4))



