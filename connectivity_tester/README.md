# MongoDB ODBC Connectivity Tester

A standalone tool for testing MongoDB ODBC connectivity with support for various authentication mechanisms. This tool is designed for support teams and customers to diagnose connection issues without requiring a full Power BI or ODBC application setup.

## Prerequisites

- **MongoDB ODBC Driver must be installed** on the system
- Rust toolchain (for building from source)

## Building

```bash
cd connectivity_tester
cargo build --release
```

The binary will be available at `target/release/connectivity_tester`

## Usage

### Test with ODBC Connection String

```bash
connectivity_tester odbc \
  --connection-string "DRIVER={MongoDB Atlas SQL ODBC Driver};USER=myuser;PWD=mypass;SERVER=localhost:27017" \
  --database mydb \
  --verbose
```

### Test with MongoDB URI

```bash
connectivity_tester uri \
  --uri "mongodb://myuser:mypass@localhost:27017/?authSource=admin" \
  --database mydb \
  --verbose
```

### View Examples

```bash
connectivity_tester examples
```

## Command-Line Options

### Common Options (both `odbc` and `uri` commands)

- `-d, --database <DATABASE>` - Database to connect to (optional, can override connection string)
- `-t, --login-timeout <SECONDS>` - Login timeout in seconds (default: 30)
- `-c, --connection-timeout <SECONDS>` - Connection timeout for operations (optional)
- `-s, --simple-types` - Use simple type mode instead of standard types
- `-m, --max-string-length` - Enable 4000 character limit (required by some BI tools like SSMS)
- `-v, --verbose` - Show detailed connection information

### ODBC Command

- `-c, --connection-string <STRING>` - Full ODBC connection string

### URI Command

- `-u, --uri <URI>` - MongoDB connection URI

## Connection String Formats

### ODBC Format

The ODBC connection string supports the following parameters:

**Required:**
- `DRIVER` - Path or name of the MongoDB ODBC driver
- Either:
  - `URI` - Full MongoDB connection URI, OR
  - `USER`, `PWD`, `SERVER` - Individual connection parameters

**Optional:**
- `DATABASE` - Database name
- `LOGLEVEL` - Log level for debugging (trace, debug, info, warn, error)
- `SIMPLE_TYPES_ONLY` - Set to "1" for simple type mode, "0" for standard (default: "1")
- `ENABLE_MAX_STRING_LENGTH` - Set to "1" to enable 4000 char limit (default: "0")

**Example:**
```
DRIVER={MongoDB Atlas SQL ODBC Driver};USER=admin;PWD=password;SERVER=localhost:27017;DATABASE=test
```

### MongoDB URI Format

Standard MongoDB connection URI with optional parameters:

**Basic format:**
```
mongodb://[username:password@]host[:port][/[defaultauthdb]][?options]
```

**SRV format (for Atlas):**
```
mongodb+srv://[username:password@]host[/[defaultauthdb]][?options]
```

## Authentication Mechanisms

### 1. SCRAM-SHA-256 (Default)

```bash
connectivity_tester uri \
  --uri "mongodb://user:pass@localhost:27017/?authSource=admin"
```

### 2. SCRAM-SHA-1

```bash
connectivity_tester uri \
  --uri "mongodb://user:pass@localhost:27017/?authMechanism=SCRAM-SHA-1&authSource=admin"
```

### 3. X.509 Certificate

```bash
connectivity_tester uri \
  --uri "mongodb://localhost:27017/?authMechanism=MONGODB-X509&authSource=\$external&tls=true&tlsCertificateKeyFile=/path/to/cert.pem"
```

**Note:** Username and password are not required in the URI for X.509 authentication. The tool automatically adds dummy credentials internally for validation purposes, which are then cleared by the ODBC driver for X.509 connections (as per the driver's design).

### 4. LDAP (PLAIN)

```bash
connectivity_tester uri \
  --uri "mongodb://ldapuser:ldappass@localhost:27017/?authMechanism=PLAIN&authSource=\$external"
```

### 5. AWS IAM

```bash
connectivity_tester uri \
  --uri "mongodb://localhost:27017/?authMechanism=MONGODB-AWS&authSource=\$external"
```

Note: AWS credentials are typically obtained from environment variables or IAM role.

### 6. Kerberos (GSSAPI)

```bash
connectivity_tester uri \
  --uri "mongodb://user@REALM@localhost:27017/?authMechanism=GSSAPI&authSource=\$external"
```

Note: Not available on Windows.

### 7. OpenID Connect (OIDC)

```bash
connectivity_tester uri \
  --uri "mongodb://localhost:27017/?authMechanism=MONGODB-OIDC&authSource=\$external"
```

Note: This will open a browser window for the authentication flow.

## Common URI Parameters

- `?authSource=admin` - Authentication database (default: admin, use $external for X.509, LDAP, AWS, OIDC, Kerberos)
- `&authMechanism=SCRAM-SHA-256` - Authentication mechanism
- `&tls=true` - Enable TLS/SSL
- `&tlsAllowInvalidCertificates=true` - Allow self-signed certificates (not recommended for production)
- `&tlsCAFile=/path/to/ca.pem` - Path to CA certificate file
- `&tlsCertificateKeyFile=/path/to/cert.pem` - Path to client certificate
- `&retryWrites=true` - Enable retry writes
- `&w=majority` - Write concern
- `&uuidRepresentation=standard` - UUID representation (standard, csharpLegacy, javaLegacy, pythonLegacy)

## Troubleshooting

The tool provides context-aware troubleshooting tips based on the error encountered:

- **Authentication errors** - Suggests checking credentials, authSource, and authMechanism
- **Timeout errors** - Suggests network connectivity checks and timeout adjustments
- **DNS errors** - Provides DNS resolution tips (Windows uses Cloudflare DNS by default)
- **TLS/SSL errors** - Suggests certificate configuration options
- **Database errors** - Reminds to specify database parameter
- **Connection refused** - Suggests checking server status and firewall rules

## Output

The tool provides step-by-step feedback:

1. ✓ Connection string parsing
2. ✓ Runtime creation
3. ✓ Client options parsing (with auth details in verbose mode)
4. ✓ Connection establishment

On success, displays:
- Time taken to connect
- Cluster type (AtlasDataFederation or Enterprise)
- UUID representation (if configured)

On failure, displays:
- Error details
- Time taken before failure
- Context-aware troubleshooting tips

## Examples

See all examples:
```bash
connectivity_tester examples
```

## Notes

- The tool uses the same connection logic as the full ODBC driver
- Requires the MongoDB ODBC driver to be installed (but not configured in ODBC manager)
- Default login timeout is 30 seconds
- On Windows, DNS resolution uses Cloudflare DNS by default with fallback to system resolver
- The tool performs a `SELECT 1` query to verify the connection is fully functional

