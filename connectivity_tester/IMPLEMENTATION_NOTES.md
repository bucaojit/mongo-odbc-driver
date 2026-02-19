# Implementation Notes - MongoDB ODBC Connectivity Tester

## Overview

This connectivity tester is a standalone command-line tool designed to help support teams and customers diagnose MongoDB ODBC connection issues without requiring a full ODBC application setup (like Power BI).

## Key Features

### 1. **Dual Connection String Support**
- **ODBC Format**: Traditional ODBC connection strings with DRIVER, USER, PWD, SERVER parameters
- **MongoDB URI Format**: Native MongoDB connection URIs (mongodb:// or mongodb+srv://)

### 2. **Comprehensive Authentication Support**
The tool supports all MongoDB authentication mechanisms:
- SCRAM-SHA-1 and SCRAM-SHA-256 (default)
- X.509 Certificate Authentication
- LDAP (PLAIN mechanism)
- AWS IAM Authentication
- Kerberos (GSSAPI) - not available on Windows
- OpenID Connect (OIDC) with browser-based flow

### 3. **Important Configuration Options**

Based on code analysis, the following options are critical for compatibility:

#### Type Mode
- **Standard** (default): Full BSON type support
- **Simple**: Simplified type mapping for compatibility with certain tools
- Controlled via `--simple-types` flag or `SIMPLE_TYPES_ONLY=1` in connection string

#### Max String Length
- Some BI tools (like Microsoft SQL Server Management Studio) require a maximum string length
- Default: unlimited
- When enabled: 4000 characters (industry standard for compatibility)
- Controlled via `--max-string-length` flag or `ENABLE_MAX_STRING_LENGTH=1` in connection string

#### Timeouts
- **Login Timeout**: Time to wait for initial connection (default: 30s)
- **Connection Timeout**: Time to wait for operations after connection
- Critical for slow networks or Atlas connections

#### Database Selection
- Required for connection validation
- Can be specified in URI, connection string, or via `--database` flag
- The `--database` flag overrides other sources

### 4. **Gotchas and Edge Cases**

#### DNS Resolution (Windows-specific)
- On Windows, the driver uses Cloudflare DNS by default for better performance
- Automatically falls back to system resolver if Cloudflare DNS fails
- This is handled transparently in `core/src/odbc_uri.rs` lines 467-483

#### Authentication Source
- Default auth source is `admin` for standard authentication
- Must use `$external` for: X.509, LDAP, AWS IAM, OIDC, Kerberos
- Common mistake: using wrong authSource for the authentication mechanism

#### Special Characters in Passwords
- Must be URL-encoded in MongoDB URIs
- ODBC connection strings handle this differently
- The tool uses the same parsing logic as the driver

#### OIDC Authentication Flow
- Opens browser window for authentication
- Uses local HTTP server on port 27097 for redirect
- Supports refresh tokens for subsequent authentications
- Implementation in `core/src/oidc_auth.rs`

#### X.509 Authentication
- Username and password are cleared for X.509 (see `core/src/odbc_uri.rs` lines 314-316)
- The connectivity tester adds dummy USER and PWD when using the `uri` command because `ODBCUri` requires them for validation
- These dummy credentials are automatically cleared by the driver for X.509 authentication
- Requires TLS to be enabled
- Certificate paths must be absolute

#### Connection Validation
- The tool performs a `SELECT 1` query to verify the connection is fully functional
- This is the same validation used by the ODBC driver
- Ensures not just network connectivity but also database access

### 5. **Error Handling and Diagnostics**

The tool provides context-aware troubleshooting tips based on error patterns:
- Authentication errors → credential and mechanism suggestions
- Timeout errors → network and firewall checks
- DNS errors → resolution and hostname tips
- TLS/SSL errors → certificate configuration guidance
- Database errors → access and specification reminders
- Connection refused → server status and firewall checks

### 6. **Code Structure**

```
connectivity_tester/
├── Cargo.toml           # Dependencies
├── src/
│   └── main.rs          # Main application logic
├── README.md            # Comprehensive documentation
├── QUICK_START.md       # Quick reference for support teams
└── IMPLEMENTATION_NOTES.md  # This file
```

### 7. **Dependencies**

Key dependencies:
- `mongo-odbc-core`: Core ODBC driver logic (reuses existing connection code)
- `constants`: Shared constants (DEFAULT_MAX_STRING_LENGTH, etc.)
- `clap`: Command-line argument parsing
- `tokio`: Async runtime (required by MongoDB driver)
- `colored`: Terminal output formatting

### 8. **Building and Distribution**

#### Prerequisites
- Rust toolchain
- MongoDB ODBC driver installed on the system

#### Build Command
```bash
cargo build --release
```

#### Binary Location
```
target/release/connectivity_tester
```

#### Platform-Specific Notes
- **macOS**: Requires iODBC or unixODBC
- **Linux**: Requires unixODBC
- **Windows**: Uses Windows ODBC Driver Manager

The tool does NOT require:
- ODBC Data Source (DSN) configuration
- ODBC Driver Manager registration (in some cases)

But it DOES require:
- MongoDB ODBC driver libraries to be installed
- Proper library paths for linking

### 9. **Testing Recommendations**

When testing the tool, verify:
1. ✓ ODBC connection string parsing
2. ✓ MongoDB URI parsing
3. ✓ Authentication mechanism detection
4. ✓ Timeout handling
5. ✓ Error message clarity
6. ✓ Verbose output completeness
7. ✓ Examples command output

### 10. **Future Enhancements**

Potential improvements:
- JSON output mode for programmatic use
- Connection pooling tests
- Performance benchmarking
- SSL/TLS certificate validation testing
- Batch testing from configuration file
- Integration with monitoring systems

### 11. **Key Code References**

Important files to understand:
- `core/src/odbc_uri.rs`: Connection string parsing and client options
- `core/src/conn.rs`: Connection establishment logic
- `core/src/util/test_connection.rs`: Existing test connection function
- `odbc/src/api/functions.rs`: ODBC API implementation
- `core/src/oidc_auth.rs`: OIDC authentication flow

### 12. **Connection String Parameters**

All supported ODBC connection string parameters:
- `DRIVER` - Driver name or path
- `URI` - MongoDB connection URI
- `USER` / `UID` - Username
- `PWD` / `PASSWORD` - Password
- `SERVER` - Server address and port
- `DATABASE` - Database name
- `LOGLEVEL` - Logging level (trace, debug, info, warn, error)
- `SIMPLE_TYPES_ONLY` - "1" for simple types, "0" for standard
- `ENABLE_MAX_STRING_LENGTH` - "1" to enable 4000 char limit
- `APPNAME` - Application name for telemetry

### 13. **MongoDB URI Parameters**

Common URI parameters:
- `?authSource=admin` - Authentication database
- `&authMechanism=SCRAM-SHA-256` - Auth mechanism
- `&tls=true` - Enable TLS/SSL
- `&tlsAllowInvalidCertificates=true` - Allow self-signed certs
- `&tlsCAFile=/path/to/ca.pem` - CA certificate
- `&tlsCertificateKeyFile=/path/to/cert.pem` - Client certificate
- `&retryWrites=true` - Enable retry writes
- `&w=majority` - Write concern
- `&uuidRepresentation=standard` - UUID representation

## Conclusion

This tool provides a comprehensive, self-contained way to test MongoDB ODBC connectivity. It reuses the core driver logic to ensure test results accurately reflect what the actual ODBC driver will experience, making it an invaluable diagnostic tool for support and customers.

