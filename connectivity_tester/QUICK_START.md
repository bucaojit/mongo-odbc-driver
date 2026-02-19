# Quick Start Guide - MongoDB ODBC Connectivity Tester

## For Support Teams

When a customer reports they cannot connect through Power BI or other ODBC applications, use this tool to isolate the issue.

### Step 1: Get the Connection Information

Ask the customer for their connection details. They can provide either:

**Option A: ODBC Connection String**
```
DRIVER={MongoDB Atlas SQL ODBC Driver};USER=username;PWD=password;SERVER=host:port;DATABASE=dbname
```

**Option B: MongoDB URI**
```
mongodb://username:password@host:port/?authSource=admin
```

### Step 2: Run the Tester

**With ODBC connection string:**
```bash
./connectivity_tester odbc \
  --connection-string "DRIVER={MongoDB Atlas SQL ODBC Driver};USER=username;PWD=password;SERVER=host:port" \
  --database dbname \
  --verbose
```

**With MongoDB URI:**
```bash
./connectivity_tester uri \
  --uri "mongodb://username:password@host:port/?authSource=admin" \
  --database dbname \
  --verbose
```

### Step 3: Interpret Results

#### ✓ Success
If the test passes, the issue is likely with the ODBC application configuration, not the connection itself.

**Next steps:**
- Verify ODBC driver is properly registered in the system
- Check ODBC Data Source configuration
- Review application-specific ODBC settings

#### ✗ Failure
The tool will provide specific troubleshooting tips based on the error.

**Common issues:**

1. **Authentication Failed**
   - Wrong username/password
   - Wrong authSource (try `admin` or `$external`)
   - Wrong authMechanism for the deployment

2. **Timeout**
   - Network connectivity issue
   - Firewall blocking connection
   - Wrong host/port
   - For Atlas: IP not whitelisted

3. **DNS Resolution Failed**
   - Hostname cannot be resolved
   - Try using IP address instead
   - Check DNS configuration

4. **TLS/SSL Error**
   - Missing `?tls=true` in URI
   - Certificate validation issues
   - Try `?tlsAllowInvalidCertificates=true` for testing

## For Customers

### Installation

1. Ensure MongoDB ODBC driver is installed
2. Download the connectivity tester binary for your platform
3. Make it executable (Linux/Mac): `chmod +x connectivity_tester`

### Basic Test

The simplest way to test your connection:

```bash
# Using ODBC format
./connectivity_tester odbc \
  -c "DRIVER={MongoDB Atlas SQL ODBC Driver};USER=myuser;PWD=mypass;SERVER=myhost:27017" \
  -d mydatabase

# Using MongoDB URI
./connectivity_tester uri \
  -u "mongodb://myuser:mypass@myhost:27017/?authSource=admin" \
  -d mydatabase
```

### See Examples

To see examples for different authentication types:

```bash
./connectivity_tester examples
```

### Common Scenarios

#### Testing Atlas Connection

```bash
./connectivity_tester uri \
  -u "mongodb+srv://username:password@cluster.mongodb.net/?authSource=admin" \
  -d sample_mflix \
  --verbose
```

#### Testing with X.509 Certificate

```bash
./connectivity_tester uri \
  -u "mongodb://host:27017/?authMechanism=MONGODB-X509&authSource=\$external&tls=true&tlsCertificateKeyFile=/path/to/cert.pem" \
  -d mydb
```

#### Testing with Increased Timeout

If you have a slow network:

```bash
./connectivity_tester uri \
  -u "mongodb://user:pass@host:27017/" \
  -d mydb \
  --login-timeout 60
```

#### Testing with Power BI Compatible Settings

Power BI requires specific settings:

```bash
./connectivity_tester odbc \
  -c "DRIVER={MongoDB Atlas SQL ODBC Driver};USER=user;PWD=pass;SERVER=host:27017" \
  -d mydb \
  --max-string-length \
  --verbose
```

## Gotchas and Important Notes

### 1. Driver Must Be Installed
The connectivity tester requires the MongoDB ODBC driver to be installed on the system, but it does NOT require:
- ODBC Data Source (DSN) configuration
- ODBC Driver Manager registration (in some cases)

### 2. Authentication Source
- For standard MongoDB authentication, use `?authSource=admin`
- For X.509, LDAP, AWS IAM, OIDC, Kerberos, use `?authSource=$external`

### 3. Special Characters in Passwords
If your password contains special characters, they must be URL-encoded in the URI:
- `@` → `%40`
- `:` → `%3A`
- `/` → `%2F`
- `?` → `%3F`
- `#` → `%23`
- `&` → `%26`

Example:
```bash
# Password: p@ss:word
mongodb://user:p%40ss%3Aword@host:27017/
```

### 4. Windows DNS Resolution
On Windows, the driver uses Cloudflare DNS by default for better performance. If this causes issues, it will automatically fall back to the system resolver.

### 5. OIDC Authentication
When using OIDC (`authMechanism=MONGODB-OIDC`), the tool will:
- Open your default web browser
- Prompt you to authenticate
- Wait for the authentication flow to complete

### 6. Type Modes
- **Standard types** (default): Full BSON type support
- **Simple types** (`--simple-types`): Simplified type mapping for compatibility

Most users should use standard types unless they have specific compatibility requirements.

### 7. Max String Length
Some BI tools (like Microsoft SQL Server Management Studio) require a maximum string length to be specified. Use `--max-string-length` to enable a 4000 character limit.

### 8. Database Parameter
While the database can be specified in the URI or connection string, using the `--database` flag will override it. This is useful for testing access to different databases with the same credentials.

## Troubleshooting Tips

### Connection Hangs
- Increase timeout: `--login-timeout 60`
- Check firewall rules
- Verify server is accessible: `ping hostname` or `telnet hostname 27017`

### "No Database" Error
- Always specify a database with `-d` or `DATABASE=` in connection string
- Verify the database exists and user has access

### Certificate Errors
- Ensure certificate paths are absolute
- Check file permissions on certificate files
- For testing, try `?tlsAllowInvalidCertificates=true`

### Authentication Errors
- Verify credentials are correct
- Check if user exists: `db.getUsers()` in mongo shell
- Verify user has appropriate roles
- Ensure authSource is correct

## Getting Help

If the connectivity tester succeeds but your application still fails to connect:
1. The issue is likely with the application's ODBC configuration
2. Check the application's ODBC settings
3. Verify the ODBC driver is properly registered
4. Review application-specific documentation

If the connectivity tester fails:
1. Review the troubleshooting tips provided by the tool
2. Run with `--verbose` for more details
3. Check MongoDB server logs
4. Verify network connectivity

