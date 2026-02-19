# MongoDB ODBC Connectivity Tester - Architecture

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Connectivity Tester CLI                       │
│                         (main.rs)                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Uses
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    mongo-odbc-core Library                       │
│                  (Same as ODBC Driver)                           │
├─────────────────────────────────────────────────────────────────┤
│  • ODBCUri - Connection string parsing                          │
│  • MongoConnection - Connection establishment                   │
│  • Authentication - All auth mechanisms                         │
│  • Error handling - Comprehensive error types                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Connects to
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      MongoDB Deployment                          │
│  • Atlas (mongodb+srv://)                                       │
│  • Self-hosted (mongodb://)                                     │
│  • Enterprise Server                                            │
│  • Atlas Data Federation                                        │
└─────────────────────────────────────────────────────────────────┘
```

## Component Flow

```
User Input
    │
    ├─── ODBC Connection String ──┐
    │                             │
    └─── MongoDB URI ─────────────┤
                                  │
                                  ▼
                        ┌──────────────────┐
                        │  Argument Parser │
                        │     (clap)       │
                        └──────────────────┘
                                  │
                                  ▼
                        ┌──────────────────┐
                        │   ODBCUri::new   │
                        │  Parse & Validate│
                        └──────────────────┘
                                  │
                                  ▼
                        ┌──────────────────┐
                        │ Tokio Runtime    │
                        │   Creation       │
                        └──────────────────┘
                                  │
                                  ▼
                        ┌──────────────────┐
                        │ Client Options   │
                        │    Parsing       │
                        └──────────────────┘
                                  │
                                  ├─── Auth Mechanism Detection
                                  ├─── Server Resolution
                                  ├─── TLS Configuration
                                  └─── Credential Setup
                                  │
                                  ▼
                        ┌──────────────────┐
                        │ MongoConnection  │
                        │    ::connect     │
                        └──────────────────┘
                                  │
                                  ├─── Network Connection
                                  ├─── Authentication
                                  ├─── Database Selection
                                  └─── Validation Query (SELECT 1)
                                  │
                                  ▼
                        ┌──────────────────┐
                        │  Success/Failure │
                        │     Output       │
                        └──────────────────┘
                                  │
                                  ├─── Success: Connection details
                                  └─── Failure: Error + Troubleshooting tips
```

## Authentication Flow

```
Connection String/URI
        │
        ▼
┌───────────────────────────────────────────────────────────┐
│              Authentication Mechanism Detection            │
├───────────────────────────────────────────────────────────┤
│  • Check authMechanism parameter in URI                   │
│  • Default to SCRAM-SHA-256 if not specified              │
│  • Validate credentials for mechanism                     │
└───────────────────────────────────────────────────────────┘
        │
        ├─── SCRAM-SHA-1/256 ──────┐
        ├─── MONGODB-X509 ──────────┤
        ├─── PLAIN (LDAP) ──────────┤
        ├─── MONGODB-AWS ───────────┤
        ├─── GSSAPI (Kerberos) ─────┤
        └─── MONGODB-OIDC ──────────┤
                                    │
                                    ▼
                        ┌──────────────────────┐
                        │  Credential Setup    │
                        ├──────────────────────┤
                        │ • Username/Password  │
                        │ • Certificates       │
                        │ • Tokens             │
                        │ • Auth Source        │
                        └──────────────────────┘
                                    │
                                    ▼
                        ┌──────────────────────┐
                        │   Authentication     │
                        │   with MongoDB       │
                        └──────────────────────┘
```

## OIDC Authentication Flow (Special Case)

```
OIDC Connection Request
        │
        ▼
┌─────────────────────────┐
│  Start Local HTTP       │
│  Server (port 27097)    │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  Open Browser with      │
│  Authorization URL      │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  User Authenticates     │
│  with Identity Provider │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  Redirect to Local      │
│  HTTP Server            │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  Exchange Code for      │
│  Access Token           │
└─────────────────────────┘
        │
        ▼
┌─────────────────────────┐
│  Connect to MongoDB     │
│  with Access Token      │
└─────────────────────────┘
```

## Error Handling Flow

```
Error Occurs
    │
    ▼
┌─────────────────────────────────────────┐
│        Error Type Detection             │
├─────────────────────────────────────────┤
│  • Authentication Error                 │
│  • Timeout Error                        │
│  • DNS Resolution Error                 │
│  • TLS/SSL Error                        │
│  • Database Error                       │
│  • Connection Refused                   │
│  • Other Errors                         │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│   Context-Aware Troubleshooting Tips    │
├─────────────────────────────────────────┤
│  • Specific to error type               │
│  • Actionable suggestions               │
│  • Common solutions                     │
│  • Example configurations               │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│         Formatted Error Output          │
├─────────────────────────────────────────┤
│  • Error message                        │
│  • Time taken                           │
│  • Troubleshooting tips                 │
│  • Example connection strings           │
└─────────────────────────────────────────┘
```

## Data Flow

```
┌──────────────────┐
│  User Input      │
│  • Connection    │
│    String        │
│  • Flags         │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Parsed Config   │
│  • URI/ODBC      │
│  • Database      │
│  • Timeouts      │
│  • Type Mode     │
│  • Max Length    │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Client Options  │
│  • Hosts         │
│  • Credentials   │
│  • Auth Mech     │
│  • TLS Config    │
│  • App Name      │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Connection      │
│  • Client        │
│  • Runtime       │
│  • Cluster Type  │
│  • UUID Repr     │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│  Output          │
│  • Success/Fail  │
│  • Details       │
│  • Timing        │
│  • Tips          │
└──────────────────┘
```

## Module Dependencies

```
connectivity_tester
    │
    ├── clap (CLI parsing)
    ├── colored (Terminal output)
    ├── tokio (Async runtime)
    │
    └── mongo-odbc-core
            │
            ├── mongodb (MongoDB driver)
            │   ├── bson
            │   ├── tokio
            │   └── Authentication libraries
            │       ├── aws-config (AWS IAM)
            │       ├── openidconnect (OIDC)
            │       ├── cross-krb5 (Kerberos)
            │       └── libgssapi (GSSAPI)
            │
            ├── constants (Shared constants)
            ├── cstr (String utilities)
            ├── definitions (ODBC definitions)
            └── shared_sql_utils (DSN utilities)
```

## Key Design Decisions

### 1. Reuse Core Library
**Decision**: Use `mongo-odbc-core` library directly
**Rationale**: 
- Ensures test results match actual driver behavior
- Avoids code duplication
- Maintains consistency with driver updates

### 2. Step-by-Step Validation
**Decision**: Show progress for each connection step
**Rationale**:
- Helps identify exactly where connection fails
- Provides better user experience
- Easier troubleshooting

### 3. Context-Aware Error Messages
**Decision**: Analyze error messages and provide specific tips
**Rationale**:
- Reduces support burden
- Helps users self-diagnose
- Improves user experience

### 4. Dual Input Methods
**Decision**: Support both ODBC and MongoDB URI formats
**Rationale**:
- Flexibility for different user preferences
- Compatibility with existing configurations
- Easier migration between formats

### 5. Verbose Mode
**Decision**: Optional detailed output
**Rationale**:
- Default output is clean and simple
- Verbose mode for debugging
- Shows authentication details safely

## Security Considerations

### Password Handling
- Passwords are never logged in non-verbose mode
- Verbose mode shows `***` for passwords
- Passwords are cleared from memory after use

### Certificate Handling
- Certificate paths are validated
- Absolute paths required
- File permissions checked by driver

### OIDC Tokens
- Tokens are ephemeral
- Refresh tokens supported
- Tokens not persisted to disk

## Performance Characteristics

### Connection Time
- Typical: 1-3 seconds for local connections
- Atlas: 2-5 seconds depending on network
- OIDC: 30-60 seconds (includes user interaction)

### Resource Usage
- Memory: ~50MB (includes MongoDB driver)
- CPU: Minimal (mostly I/O bound)
- Network: Single connection, minimal traffic

### Timeouts
- Default login timeout: 30 seconds
- Configurable via `--login-timeout`
- Connection timeout optional

## Extensibility

### Adding New Authentication Mechanisms
1. Update `mongo-odbc-core` library
2. Add example to `show_examples()`
3. Update documentation

### Adding New Options
1. Add to `Cli` struct
2. Pass to connection logic
3. Update documentation

### Adding New Output Formats
1. Add new command or flag
2. Implement formatter
3. Update examples

## Testing Strategy

### Unit Tests
- Connection string parsing
- Error message generation
- Argument parsing

### Integration Tests
- Actual connection tests
- Authentication mechanism tests
- Error handling tests

### Manual Tests
- Different MongoDB versions
- Different authentication mechanisms
- Different network conditions
- Different platforms (Windows, macOS, Linux)

