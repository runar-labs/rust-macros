# Changelog

## [Unreleased]

### Added
- Comprehensive documentation for all macros with usage examples
- New `IMPLEMENTATION_PLAN.md` document outlining the roadmap for macros
- New `DEBUGGING.md` guide with practical tips for troubleshooting macros
- Enhanced test coverage for the service macro with more test cases
- Improved doc comments for IDE support

### Fixed
- Updated service macro to work with syn 2.0
- Fixed attribute parsing with custom `MacroArgs` parser
- Corrected method names in generated code (`service_name`, `service_path`, etc.)
- Added proper handling of default values for service properties
- Added leading slash to default paths for consistency
- Implemented runtime registration system as alternative to distributed slices
- Fixed `handle_request` method implementation in the service registry
- Added proper error handling for unimplemented operations
- Resolved issues with P2P delegate and message handling
- Fixed parameter processing in subscription methods
- Fixed `unsubscribe` method implementation to handle subscription IDs correctly

### Changed
- Improved error handling with better error messages
- Enhanced debug output for macro execution
- Updated minimal service test with more test cases and better documentation
- Refactored distributed slice pattern to support runtime registration as fallback
- Added complete end-to-end tests for macro functionality validation

## [0.1.0]

### Added
- Initial implementation of macros:
  - `service` for defining Kagi services
  - `action` for defining service operations
  - `process` for request processing
  - `subscribe` for event handling
  - `publish` for event publishing 