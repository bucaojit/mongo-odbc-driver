# X.509 Authentication Fix

## Problem

When using the `uri` command with X.509 authentication, the connectivity tester was failing with errors like:

```
Error: Invalid Uri: One of ["uid", "user"] is required for a valid Mongo ODBC Uri
```

or

```
Error: Invalid Uri: One of ["password", "pwd"] is required for a valid Mongo ODBC Uri
```

## Root Cause

The MongoDB ODBC driver's `ODBCUri` class requires `USER` and `PWD` attributes to be present in the connection string for validation purposes, even for authentication mechanisms like X.509 that don't use username/password credentials.

### How the ODBC Driver Handles This

In the actual ODBC driver, when applications connect using X.509:

1. The application provides a connection string like:
   ```
   DRIVER={MongoDB Atlas SQL ODBC Driver};URI=mongodb://...?authMechanism=MONGODB-X509;USER=foo;PWD=bar
   ```

2. The `ODBCUri::new()` function validates that USER and PWD are present

3. After validation, the driver detects the X.509 authentication mechanism and clears the credentials:
   ```rust
   // From core/src/odbc_uri.rs lines 314-316
   Some(AuthMechanism::MongoDbX509) => {
       opts.credential.as_mut().unwrap().username = None;
       opts.credential.as_mut().unwrap().password = None;
   }
   ```

### Why the Connectivity Tester Failed

The connectivity tester's `uri` command was converting MongoDB URIs to ODBC connection strings like this:

```rust
// OLD CODE (BROKEN)
let connection_string = format!("URI={}", uri);
```

This resulted in connection strings without USER and PWD attributes, causing validation to fail before the X.509 credential clearing logic could run.

## Solution

The fix adds dummy USER and PWD attributes when using the `uri` command:

```rust
// NEW CODE (FIXED)
// Note: We add dummy USER and PWD here because ODBCUri requires them
// for validation, but they will be cleared for X.509 and other mechanisms
// that don't use username/password (see core/src/odbc_uri.rs lines 314-316)
let connection_string = format!("URI={};USER=dummy;PWD=dummy", uri);
```

This matches the pattern used in the driver's own tests (see `core/src/odbc_uri.rs` lines 655-661):

```rust
#[test]
fn test_x509_does_not_modify_uri() {
    use crate::odbc_uri::ODBCUri;
    let uri = "mongodb://localhost:27017/abc?authSource=$external&authMechanism=MONGODB-X509";
    let mut odbc_uri = ODBCUri::new(format!("URI={uri};User=foo;PWD=bar")).unwrap();
    assert_eq!(odbc_uri.construct_uri_for_parsing(uri).unwrap(), uri);
}
```

## Impact

This fix allows X.509 authentication to work correctly with the `uri` command:

### Before (Broken)
```bash
$ connectivity_tester uri \
  -u "mongodb+srv://cluster.mongodb.net/?authMechanism=MONGODB-X509&tls=true&tlsCertificateKeyFile=/path/to/cert.pem" \
  -d mydb

Error: Invalid Uri: One of ["uid", "user"] is required for a valid Mongo ODBC Uri
```

### After (Fixed)
```bash
$ connectivity_tester uri \
  -u "mongodb+srv://cluster.mongodb.net/?authMechanism=MONGODB-X509&tls=true&tlsCertificateKeyFile=/path/to/cert.pem" \
  -d mydb

✓ Connection string parsed successfully
✓ Runtime created successfully
✓ Client options parsed successfully
✓ Connection established successfully
```

## Other Authentication Mechanisms Affected

This fix also benefits other authentication mechanisms that don't require username/password in the URI:

1. **MONGODB-OIDC** - OpenID Connect authentication
   - The driver clears empty username/password for OIDC (lines 324-334 in `core/src/odbc_uri.rs`)
   - Dummy credentials are added and then cleared

2. **MONGODB-AWS** - AWS IAM authentication
   - Credentials come from AWS environment/IAM role
   - Dummy credentials are added for validation but not used

For mechanisms that DO require username/password (SCRAM, LDAP, Kerberos), the dummy credentials are overridden by the actual credentials in the URI, so there's no negative impact.

## Testing

The fix has been tested with:
- ✅ X.509 authentication (primary use case)
- ✅ SCRAM-SHA-256 authentication (no regression)
- ✅ Code compiles successfully (`cargo check`)

## Documentation Updates

The following documentation files have been updated to explain this behavior:

1. **README.md** - Added note explaining that dummy credentials are added internally for X.509
2. **IMPLEMENTATION_NOTES.md** - Added technical details about the dummy credential handling
3. **X509_FIX.md** - This file, explaining the fix in detail

## References

- `core/src/odbc_uri.rs` lines 314-316: X.509 credential clearing
- `core/src/odbc_uri.rs` lines 655-661: Test showing USER/PWD required for X.509
- `core/src/odbc_uri.rs` lines 279-290: Validation requiring USER and PWD
- `connectivity_tester/src/main.rs` lines 114-117: The fix implementation

