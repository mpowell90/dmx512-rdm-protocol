# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.3.1...v0.4.0) - 2024-09-28

### Added

- no_std support

### Other

- fix doc test bug
- use interleaving cfg flag implementation for StatusMessage
- rust fmt fixes
- clearer no_std messaging in README
- better doc examples

## [0.3.1](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.3.0...v0.3.1) - 2024-09-28

### Added

- added impl for core::error::Error

### Other

- updated msrv docs

## [0.3.0](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.2.2...v0.3.0) - 2024-08-16

### Fixed
- strings might not contain null characters, terminate early if they do

### Other
- removed imports and redundant error variant, added tests

## [0.2.2](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.2.1...v0.2.2) - 2024-08-16

### Added
- added TryFrom<Vec<u8>> impl, changed current impl to not work on dmx packets, improved tests

## [0.2.1](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.2.0...v0.2.1) - 2024-08-15

### Fixed
- expose inner DeviceUID, added Copy derive

## [0.2.0](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.1.1...v0.2.0) - 2024-08-15

### Added
- added extend method to DmxUniverse

### Other
- updated DmxUniverse tests and README.md
- changes as_bytes to as_slice

## [0.1.1](https://github.com/mpowell90/dmx512-rdm-protocol/compare/v0.1.0...v0.1.1) - 2024-08-13

### Other
- license instead of license-file
- add reference to crates.io for dependency version
- added badges and removed dependency installation example from README.md

## [0.1.0](https://github.com/mpowell90/dmx512-rdm-protocol/releases/tag/v0.1.0) - 2024-08-13

### Added
- handle ManufacturerSpecific LampState
- handle ManufacturerSpecific LampOnMode variants
- handle ManufacturerSpecific SensorType variants
- handle ManufacturerSpecific SensorUnit variants
- handle unsupported request and response parameters
- added Display impl for ResponseNackReasonCode
- use SelfTest in request and response
- Added SubDeviceId data-type
- added TryFrom and From impl for DmxUniverse
- added Dmx Frame encode and decode functions
- added manufacturer specific parameter request and response handling, fix parameter data parsing bug
- added From impl for Request
- added MIT License
- updated .gitignore with auto-generated spec
- added set all channel values
- added response time handling, fixed bug in checksum handling
- added status message definitions, slot id definitions, slot types, improved parameter impls
- added PresetPlaybackMode, used in request and response parameter data
- use PowerState for request parameter data
- use ResetDeviceMode for request parameter data
- made rdm available under feature flag, included by default
- added DmxUniverse struct, moved rdm functionality into folder
- added RealTimeClock get response
- added DisplayLevel, PanInvert, TiltInvert and PanTiltSwap get responses
- added language and language_capabilities get response
- added BootSoftwareVersionId and BootSoftwareVersionLabel get response
- added default slot value get response
- added sensor value get response
- added SubDeviceStatusReportThreshold get response
- added status id description get response
- added status message impl
- added comms_status get response parameter data
- handle buffer advancement when parsing, better error handling
- pretty print devices
- Removed commented out code
- working rdm_codec implementation
- WIP rdm_codec
- WIP: updated main to use async tokio and new codec
- better handling of request and response frames by using enums
- Added output directory to gitignore
- Initial working EnttecDmxUsbPro Codec
- Added From trait for ResetType enum
- Added support for more PIDs featured on the test devices
- Added print fn for Device to improve output readability in console
- Added support for discovery multiple devices within a single branch
- Added support for more parameters
- Added sub_device handling along with more pids
- Handling responses for more parameters
- Added more parameter handlers
- Added checksum validation and addition rdm parameter handling
- Added serialport read packet concatention when received over multiple packets
- Added thiserror module for better error handling
- Added ansi terminal colour logging for easier reading
- Refactored Response impl, added more ParameterId variants, handling more responses in main
- correctly parse DeviceInfoResponse and add data to device in devices Hashmap
- Added mpsc channels to send data between serialport threads, successful send and parse discovery and rdm packet types
- Added create_packet and send_rdm_packet functionality to enttec driver
- WIP added many items, including Request and Response types with create and parse packet capabilities
- Initial enttec-dmx-usb-pro driver that sends DMX packets
- Initial commit with .gitignore, cargo.toml and vscode live debugger config

### Fixed
- maximum of 5 keywords in Cargo.toml
- removed Cargo.lock from .gitignore
- channel offset bug in set_channel_values
- request encoding of LampState and LampOnMode
- failing test
- Added missing parameter handlers
- correct parameter data parsing in RdmCodec encode for DiscoveryRequest. removed u48 dependency
- Removed redundant mod declarations

### Other
- release
- release
- added github workflows for pull request and release pr
- rearranged Crgo.toml props
- use license-file prop in Cargo.toml
- moved example docs to modules
- added description, keywords, readme, repo and license to Cargo.toml
- reworked error types to not impl Error, removed this_error crate, no use of std in crate
- update crate name
- updated README project name, updated examples
- changed project name
- improved DeviceUID constructors, added constants to impl block
- handle unsupported SlotType, replace TryFrom impl with From impl
- handle unsupported SlotIdDefinition, replace TryFrom impl with From impl
- removed redundant RdmError variants
- Replaced Unsupported with Unknown where relevant
- handle unsupported ProductCategory, replace TryFrom impl with From impl
- handle unsupported ProductDetail, replace TryFrom impl with From impl
- handle unsupported parameter ids, replace TryFrom impl with From impl
- added From ProductCategory for u16 impl
- added ProductDetail enum and used in response
- renamed RDM start code and sub start code constants
- removed redundant DeviceUID from impls
- removed redundant const definitions
- cleaned up response tests
- Renaming ProtocolError to RdmError
- added README.md
- improved layout and byte definitions in tests
- added TryFrom impls for Response data-types
- removed println!
- Renamed response data-types, does not return redundant Option wrapper
- response accepts shared rather than exclusive reference
- added MSRV
- removed Eq on ProtocolError
- removed redundant function
- removed unwrap
- simplified ResponseParameterData enum data layout
- removed TODO
- simplified Supported Parameter data parsing, removed manufacturer specific parameter struct
- changed Sensor to SensorDefinition, used different values for data types
- added ParameterDescription type and several enums for field data types
- replaced separate error impl with protocol error
- removed more non E120 spec parameters for now
- removed Derives from sensor related enums
- consolidated files, removed redundant code
- use DisplayInvertMode enum for request parameter data
- removed Eq derives, moved FadeTimes struct
- added test for encoding get identify device
- added From<TryFromSliceError> impl
- removed Bytes dependency
- made bytes an optional dependency of rdm feature flag
- dmx uses core instead of std
- consolidated imports
- removed Eq derive where not required
- simplified request and response data handling
- reworked request impl, added support for all E1.20 parameters
- reworked SensorValue get and set response
- ParameterId and parameter data related enum are non-exhaustive
- explicitly match against remaining parameter ids in get response enum
- reordered get response parameters
- easier to understand conversion from u8 to bool
- improved parsing of certain get response parameter data
- added TryFromSliceError to ProtocolError enum
- updated more data type enums to TryFrom impl
- changed ProductCategory conversion to u16 and TryFrom impl
- added more get response parameter data tests
- replaced remaining String parsing with CStr impl
- removed returning option in favour of result error
- added some get response parameter data tests
- parse null terminated string buffers as CStr before converting to native String
- split response types into separate files
- reworked response struct
- reworked PacketType and CommandClass error impls
- moved request and response code out into separate files
- moved supported command class to parameter file
- moved ParameterId to its own file, reworked SupportedCommandClass impl
- reworked ParameterId conversion impls
- various changes and relocation of code
- updated dependencies
- converted project into a library
- removed previous encode / decode implementation in favour of RdmCodec
- removed commented out code
- removed commented out code
- merged MessageHeader and MessageDataBlock back into a single Response type
- removed unused parameter request structs
- Moved required parameter get requests into create_standard_parameter_get_request lookup fn
- Removed comments
- Updated RDM API to use request type based trait, moved parameter structs and logic to parameters
- Reworked the Request struct and its impls
- cleaned up comments
- Cleaned up main.rs, added comments
- moved RDM logic into separate dedicated module files

## [0.1.0](https://github.com/mpowell90/dmx512-rdm-protocol/releases/tag/v0.1.0) - 2024-08-13

### Added
- handle ManufacturerSpecific LampState
- handle ManufacturerSpecific LampOnMode variants
- handle ManufacturerSpecific SensorType variants
- handle ManufacturerSpecific SensorUnit variants
- handle unsupported request and response parameters
- added Display impl for ResponseNackReasonCode
- use SelfTest in request and response
- Added SubDeviceId data-type
- added TryFrom and From impl for DmxUniverse
- added Dmx Frame encode and decode functions
- added manufacturer specific parameter request and response handling, fix parameter data parsing bug
- added From impl for Request
- added MIT License
- updated .gitignore with auto-generated spec
- added set all channel values
- added response time handling, fixed bug in checksum handling
- added status message definitions, slot id definitions, slot types, improved parameter impls
- added PresetPlaybackMode, used in request and response parameter data
- use PowerState for request parameter data
- use ResetDeviceMode for request parameter data
- made rdm available under feature flag, included by default
- added DmxUniverse struct, moved rdm functionality into folder
- added RealTimeClock get response
- added DisplayLevel, PanInvert, TiltInvert and PanTiltSwap get responses
- added language and language_capabilities get response
- added BootSoftwareVersionId and BootSoftwareVersionLabel get response
- added default slot value get response
- added sensor value get response
- added SubDeviceStatusReportThreshold get response
- added status id description get response
- added status message impl
- added comms_status get response parameter data
- handle buffer advancement when parsing, better error handling
- pretty print devices
- Removed commented out code
- working rdm_codec implementation
- WIP rdm_codec
- WIP: updated main to use async tokio and new codec
- better handling of request and response frames by using enums
- Added output directory to gitignore
- Initial working EnttecDmxUsbPro Codec
- Added From trait for ResetType enum
- Added support for more PIDs featured on the test devices
- Added print fn for Device to improve output readability in console
- Added support for discovery multiple devices within a single branch
- Added support for more parameters
- Added sub_device handling along with more pids
- Handling responses for more parameters
- Added more parameter handlers
- Added checksum validation and addition rdm parameter handling
- Added serialport read packet concatention when received over multiple packets
- Added thiserror module for better error handling
- Added ansi terminal colour logging for easier reading
- Refactored Response impl, added more ParameterId variants, handling more responses in main
- correctly parse DeviceInfoResponse and add data to device in devices Hashmap
- Added mpsc channels to send data between serialport threads, successful send and parse discovery and rdm packet types
- Added create_packet and send_rdm_packet functionality to enttec driver
- WIP added many items, including Request and Response types with create and parse packet capabilities
- Initial enttec-dmx-usb-pro driver that sends DMX packets
- Initial commit with .gitignore, cargo.toml and vscode live debugger config

### Fixed
- removed Cargo.lock from .gitignore
- channel offset bug in set_channel_values
- request encoding of LampState and LampOnMode
- failing test
- Added missing parameter handlers
- correct parameter data parsing in RdmCodec encode for DiscoveryRequest. removed u48 dependency
- Removed redundant mod declarations

### Other
- release
- added github workflows for pull request and release pr
- rearranged Crgo.toml props
- use license-file prop in Cargo.toml
- moved example docs to modules
- added description, keywords, readme, repo and license to Cargo.toml
- reworked error types to not impl Error, removed this_error crate, no use of std in crate
- update crate name
- updated README project name, updated examples
- changed project name
- improved DeviceUID constructors, added constants to impl block
- handle unsupported SlotType, replace TryFrom impl with From impl
- handle unsupported SlotIdDefinition, replace TryFrom impl with From impl
- removed redundant RdmError variants
- Replaced Unsupported with Unknown where relevant
- handle unsupported ProductCategory, replace TryFrom impl with From impl
- handle unsupported ProductDetail, replace TryFrom impl with From impl
- handle unsupported parameter ids, replace TryFrom impl with From impl
- added From ProductCategory for u16 impl
- added ProductDetail enum and used in response
- renamed RDM start code and sub start code constants
- removed redundant DeviceUID from impls
- removed redundant const definitions
- cleaned up response tests
- Renaming ProtocolError to RdmError
- added README.md
- improved layout and byte definitions in tests
- added TryFrom impls for Response data-types
- removed println!
- Renamed response data-types, does not return redundant Option wrapper
- response accepts shared rather than exclusive reference
- added MSRV
- removed Eq on ProtocolError
- removed redundant function
- removed unwrap
- simplified ResponseParameterData enum data layout
- removed TODO
- simplified Supported Parameter data parsing, removed manufacturer specific parameter struct
- changed Sensor to SensorDefinition, used different values for data types
- added ParameterDescription type and several enums for field data types
- replaced separate error impl with protocol error
- removed more non E120 spec parameters for now
- removed Derives from sensor related enums
- consolidated files, removed redundant code
- use DisplayInvertMode enum for request parameter data
- removed Eq derives, moved FadeTimes struct
- added test for encoding get identify device
- added From<TryFromSliceError> impl
- removed Bytes dependency
- made bytes an optional dependency of rdm feature flag
- dmx uses core instead of std
- consolidated imports
- removed Eq derive where not required
- simplified request and response data handling
- reworked request impl, added support for all E1.20 parameters
- reworked SensorValue get and set response
- ParameterId and parameter data related enum are non-exhaustive
- explicitly match against remaining parameter ids in get response enum
- reordered get response parameters
- easier to understand conversion from u8 to bool
- improved parsing of certain get response parameter data
- added TryFromSliceError to ProtocolError enum
- updated more data type enums to TryFrom impl
- changed ProductCategory conversion to u16 and TryFrom impl
- added more get response parameter data tests
- replaced remaining String parsing with CStr impl
- removed returning option in favour of result error
- added some get response parameter data tests
- parse null terminated string buffers as CStr before converting to native String
- split response types into separate files
- reworked response struct
- reworked PacketType and CommandClass error impls
- moved request and response code out into separate files
- moved supported command class to parameter file
- moved ParameterId to its own file, reworked SupportedCommandClass impl
- reworked ParameterId conversion impls
- various changes and relocation of code
- updated dependencies
- converted project into a library
- removed previous encode / decode implementation in favour of RdmCodec
- removed commented out code
- removed commented out code
- merged MessageHeader and MessageDataBlock back into a single Response type
- removed unused parameter request structs
- Moved required parameter get requests into create_standard_parameter_get_request lookup fn
- Removed comments
- Updated RDM API to use request type based trait, moved parameter structs and logic to parameters
- Reworked the Request struct and its impls
- cleaned up comments
- Cleaned up main.rs, added comments
- moved RDM logic into separate dedicated module files

## [0.1.0](https://github.com/mpowell90/dmx512-rdm-protocol/releases/tag/v0.1.0) - 2024-08-13

### Added
- handle ManufacturerSpecific LampState
- handle ManufacturerSpecific LampOnMode variants
- handle ManufacturerSpecific SensorType variants
- handle ManufacturerSpecific SensorUnit variants
- handle unsupported request and response parameters
- added Display impl for ResponseNackReasonCode
- use SelfTest in request and response
- Added SubDeviceId data-type
- added TryFrom and From impl for DmxUniverse
- added Dmx Frame encode and decode functions
- added manufacturer specific parameter request and response handling, fix parameter data parsing bug
- added From impl for Request
- added MIT License
- updated .gitignore with auto-generated spec
- added set all channel values
- added response time handling, fixed bug in checksum handling
- added status message definitions, slot id definitions, slot types, improved parameter impls
- added PresetPlaybackMode, used in request and response parameter data
- use PowerState for request parameter data
- use ResetDeviceMode for request parameter data
- made rdm available under feature flag, included by default
- added DmxUniverse struct, moved rdm functionality into folder
- added RealTimeClock get response
- added DisplayLevel, PanInvert, TiltInvert and PanTiltSwap get responses
- added language and language_capabilities get response
- added BootSoftwareVersionId and BootSoftwareVersionLabel get response
- added default slot value get response
- added sensor value get response
- added SubDeviceStatusReportThreshold get response
- added status id description get response
- added status message impl
- added comms_status get response parameter data
- handle buffer advancement when parsing, better error handling
- pretty print devices
- Removed commented out code
- working rdm_codec implementation
- WIP rdm_codec
- WIP: updated main to use async tokio and new codec
- better handling of request and response frames by using enums
- Added output directory to gitignore
- Initial working EnttecDmxUsbPro Codec
- Added From trait for ResetType enum
- Added support for more PIDs featured on the test devices
- Added print fn for Device to improve output readability in console
- Added support for discovery multiple devices within a single branch
- Added support for more parameters
- Added sub_device handling along with more pids
- Handling responses for more parameters
- Added more parameter handlers
- Added checksum validation and addition rdm parameter handling
- Added serialport read packet concatention when received over multiple packets
- Added thiserror module for better error handling
- Added ansi terminal colour logging for easier reading
- Refactored Response impl, added more ParameterId variants, handling more responses in main
- correctly parse DeviceInfoResponse and add data to device in devices Hashmap
- Added mpsc channels to send data between serialport threads, successful send and parse discovery and rdm packet types
- Added create_packet and send_rdm_packet functionality to enttec driver
- WIP added many items, including Request and Response types with create and parse packet capabilities
- Initial enttec-dmx-usb-pro driver that sends DMX packets
- Initial commit with .gitignore, cargo.toml and vscode live debugger config

### Fixed
- removed Cargo.lock from .gitignore
- channel offset bug in set_channel_values
- request encoding of LampState and LampOnMode
- failing test
- Added missing parameter handlers
- correct parameter data parsing in RdmCodec encode for DiscoveryRequest. removed u48 dependency
- Removed redundant mod declarations

### Other
- added github workflows for pull request and release pr
- rearranged Crgo.toml props
- use license-file prop in Cargo.toml
- moved example docs to modules
- added description, keywords, readme, repo and license to Cargo.toml
- reworked error types to not impl Error, removed this_error crate, no use of std in crate
- update crate name
- updated README project name, updated examples
- changed project name
- improved DeviceUID constructors, added constants to impl block
- handle unsupported SlotType, replace TryFrom impl with From impl
- handle unsupported SlotIdDefinition, replace TryFrom impl with From impl
- removed redundant RdmError variants
- Replaced Unsupported with Unknown where relevant
- handle unsupported ProductCategory, replace TryFrom impl with From impl
- handle unsupported ProductDetail, replace TryFrom impl with From impl
- handle unsupported parameter ids, replace TryFrom impl with From impl
- added From ProductCategory for u16 impl
- added ProductDetail enum and used in response
- renamed RDM start code and sub start code constants
- removed redundant DeviceUID from impls
- removed redundant const definitions
- cleaned up response tests
- Renaming ProtocolError to RdmError
- added README.md
- improved layout and byte definitions in tests
- added TryFrom impls for Response data-types
- removed println!
- Renamed response data-types, does not return redundant Option wrapper
- response accepts shared rather than exclusive reference
- added MSRV
- removed Eq on ProtocolError
- removed redundant function
- removed unwrap
- simplified ResponseParameterData enum data layout
- removed TODO
- simplified Supported Parameter data parsing, removed manufacturer specific parameter struct
- changed Sensor to SensorDefinition, used different values for data types
- added ParameterDescription type and several enums for field data types
- replaced separate error impl with protocol error
- removed more non E120 spec parameters for now
- removed Derives from sensor related enums
- consolidated files, removed redundant code
- use DisplayInvertMode enum for request parameter data
- removed Eq derives, moved FadeTimes struct
- added test for encoding get identify device
- added From<TryFromSliceError> impl
- removed Bytes dependency
- made bytes an optional dependency of rdm feature flag
- dmx uses core instead of std
- consolidated imports
- removed Eq derive where not required
- simplified request and response data handling
- reworked request impl, added support for all E1.20 parameters
- reworked SensorValue get and set response
- ParameterId and parameter data related enum are non-exhaustive
- explicitly match against remaining parameter ids in get response enum
- reordered get response parameters
- easier to understand conversion from u8 to bool
- improved parsing of certain get response parameter data
- added TryFromSliceError to ProtocolError enum
- updated more data type enums to TryFrom impl
- changed ProductCategory conversion to u16 and TryFrom impl
- added more get response parameter data tests
- replaced remaining String parsing with CStr impl
- removed returning option in favour of result error
- added some get response parameter data tests
- parse null terminated string buffers as CStr before converting to native String
- split response types into separate files
- reworked response struct
- reworked PacketType and CommandClass error impls
- moved request and response code out into separate files
- moved supported command class to parameter file
- moved ParameterId to its own file, reworked SupportedCommandClass impl
- reworked ParameterId conversion impls
- various changes and relocation of code
- updated dependencies
- converted project into a library
- removed previous encode / decode implementation in favour of RdmCodec
- removed commented out code
- removed commented out code
- merged MessageHeader and MessageDataBlock back into a single Response type
- removed unused parameter request structs
- Moved required parameter get requests into create_standard_parameter_get_request lookup fn
- Removed comments
- Updated RDM API to use request type based trait, moved parameter structs and logic to parameters
- Reworked the Request struct and its impls
- cleaned up comments
- Cleaned up main.rs, added comments
- moved RDM logic into separate dedicated module files
