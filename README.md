# Authz service

## Authzed

### Schema 

You can find the schema [here](./authzed/beep.zed).
It defines the permissions of a server member. A member can have a role that has capabilities (eg: can_send_message).

### Validation

Setup the zed cli inside docker:
```bash
cd authzed
docker compose up zed-cli -d
```

Once the container is started you can enter:
```bash
docker exec -it authzed-zed-cli-1 /bin/sh
```

Move to `beep` folder inside the container:
```bash
cd beep
```

You can now launch the verification:
```bash
zed validate validations/*
```

### Repository structure

```
authzed/
├── beep.zed                           # Main SpiceDB schema definition
│                                      # Defines user, server, role, and channel resources
│                                      # with their relations and permissions
├── validations/                       # Validation test files for the schema
│   ├── send-message.yaml             # Basic send message permission tests
│   │                                  # Tests server owners and role-based permissions
│   └── permission-overrides.yaml     # Advanced permission override tests
│                                      # Tests grant/deny mechanics and precedence rules
├── docker-compose.yml                # Docker setup for SpiceDB and zed CLI
└── README.md                         # Documentation for the authzed implementation

Schema Features:
- Permission overrides (grant/deny) similar to Discord
- Role-based permissions with server ownership
- Channel-level permission controls
- Validation tests covering various scenarios
```



