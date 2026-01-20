# Authz service

## Quickstart

If you haven't set up the network yet:

```bash
docker network create authz_communities
```

**You must have a RabbitMQ instance declared with the queues specified in the [config](config/queues.json). See communities repositories for that.**

Then you should be good to start & build application:

```bash
docker compose up -d
```

**What have you done ?!**

- Started spicedb (you call it to ask for authorization) on port 50051 of your machine (this one... really important)
- Started listeners that aggregate all the data inside spicedb from community service (you should no care)

## Rust Workspace

This project is organized as a Rust workspace with the following members:

- **`listeners`**: Binary crate that listens to RabbitMQ queues for authorization events and requests
- **`core`**: Library crate that provides the core logic to interface with SpiceDB, handling permission checks and relationship management

### AuthZed gRPC Client

The `core` library includes a fully-featured Rust gRPC client for the AuthZed/SpiceDB API. See [`core/README.md`](./core/README.md) for detailed documentation.

**Quick Example:**

```rust
use core::client::AuthZedClient;

// Connect to local SpiceDB
let client = AuthZedClient::new_insecure("localhost:50051").await?;

// Or connect to AuthZed hosted
let client = AuthZedClient::new_with_token("grpc.authzed.com:443", "your_token").await?;

// Use the client
client.permissions.check_permission(request).await?;
```

For more examples, see [`core/examples/`](./core/examples/).

## Authz service

## Authzed

### Schema

You can find the schema [here](./authzed/beep.zed).
It defines the permissions of a server member. A member can have a role that has capabilities (eg: send_message).

### Capabilities Matrix

| Capability           | Server | Role | Channel | Description                       |
| -------------------- | :----: | :--: | :-----: | --------------------------------- |
| **send_message**     |   ✓    |  -   |    ✓    | Send messages in channels         |
| **view_channel**     |   ✓    |  -   |    ✓    | View and access channels          |
| **manage_message**   |   ✓    |  -   |    ✓    | Delete/edit other users' messages |
| **attach_files**     |   ✓    |  -   |    ✓    | Attach files to messages          |
| **manage_webhooks**  |   ✓    |  -   |    ✓    | Create/edit/delete webhooks       |
| **manage_role**      |   ✓    |  ✓   |    -    | Create/edit/delete roles          |
| **view_role**        |   ✓    |  ✓   |    -    | List and view role information    |
| **manage_server**    |   ✓    |  -   |    -    | Edit/delete server settings       |
| **view_server**      |   ✓    |  -   |    -    | List and view server information  |
| **manage_nicknames** |   ✓    |  -   |    -    | Edit any user's nickname          |
| **change_nickname**  |   ✓    |  -   |    -    | Change your own nickname          |

**Legend:**

- **Server**: Base permission defined at server level via role relations (e.g., `server:my_server#message_sender@role:admin#member`)
- **Role**: Entity-level overrides with grant/deny (e.g., `role:moderator#manage_role_grant@user:alice`)
- **Channel**: Entity-level overrides with grant/deny (e.g., `channel:general#send_message_grant@user:bob`)

**Permission Hierarchy:**

1. Server owner has implicit access to all capabilities
2. Server-level permissions granted through role relations
3. Entity-level grants add permissions
4. Entity-level denies remove permissions (highest priority)

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

### Role Permissions

The schema supports comprehensive role management capabilities on servers:

- **`manage_role`**: Permission to create, edit, and delete roles in the server
- **`view_role`**: Permission to list and read role information in the server

These permissions follow the same pattern as channel permissions:

- Server owners have implicit access to all role permissions
- Role-based access is granted through the `role_manager` and `role_viewer` relations
- Each permission has dedicated validation tests in `validations/roles/`

**Role-Level Permission Overrides:**

Similar to channels, roles now support permission overrides with grant/deny mechanics:

- Roles can have specific permission grants or denies for individual users or other roles
- This allows fine-grained control like "editor role can manage the moderator role specifically"
- Denies always take precedence over grants

Example usage:

```yaml
# Grant role management permission to admin role
server:my_server#role_manager@role:admin#member

# Grant role viewing permission to moderator role
server:my_server#role_viewer@role:moderator#member

# Allow editor role to manage a specific role (role-level override)
role:moderator#manage_role_grant@role:editor#member

# Allow a specific user to view a role (user-level override)
role:admin#view_role_grant@user:alice

# Deny a user from managing a specific role (deny takes precedence)
role:moderator#manage_role_deny@user:bob
```

### Server Permissions

The schema supports server management capabilities:

- **`manage_server`**: Permission to edit server settings and delete the server
- **`view_server`**: Permission to view server information and list servers
- **`manage_nicknames`**: Permission to edit any user's nickname in the server
- **`change_nickname`**: Permission to change your own nickname in the server

These permissions follow the same pattern as role permissions:

- Server owners have implicit access to all server permissions
- Role-based access is granted through the `server_manager`, `server_viewer`, `nickname_manager`, and `nickname_changer` relations
- Each permission has dedicated validation tests in `validations/servers/`

**Note:** Unlike channels and roles, servers do not support entity-level permission overrides
since the server itself is the top-level authority in the permission hierarchy.

**Nickname Permissions:**

The schema distinguishes between two types of nickname management:

- **`manage_nicknames`**: Administrative capability to edit ANY user's nickname (typically for moderators/admins)
- **`change_nickname`**: User capability to change ONLY your own nickname

These permissions are independent, allowing for scenarios where:

- Regular members can change their own nickname but not others' nicknames
- Administrators can manage all nicknames (and implicitly can change their own)
- New members might not be able to change their nickname until they gain a trusted role

Example usage:

```yaml
# Grant server management permission to admin role
server:my_server#server_manager@role:admin#member

# Grant server viewing permission to moderator role
server:my_server#server_viewer@role:moderator#member

# Grant nickname management permission to admin role
server:my_server#nickname_manager@role:admin#member

# Grant self-nickname changing permission to trusted members
server:my_server#nickname_changer@role:trusted_member#member
```

### Repository structure

```
authzed/
├── beep.zed                           # Main SpiceDB schema definition
│                                      # Defines user, server, role, and channel resources
│                                      # with their relations and permissions
├── validations/                       # Validation test files for the schema
│   ├── channels/                      # Channel permission validations
│   │   ├── send-message.yaml         # Basic send message permission tests
│   │   ├── view-channel.yaml         # Channel viewing permission tests
│   │   ├── manage-message.yaml       # Message management permission tests
│   │   ├── attach-files.yaml         # File attachment permission tests
│   │   ├── manage-webhooks.yaml      # Webhook management permission tests
│   │   └── permission-overrides.yaml # Advanced permission override tests
│   │                                  # Tests grant/deny mechanics and precedence rules
│   ├── roles/                         # Role permission validations
│   │   ├── manage-role.yaml          # Role management permission tests
│   │   ├── view-role.yaml            # Role viewing permission tests
│   │   ├── role-permissions.yaml     # Combined role permission tests
│   │   │                              # Tests permission hierarchy and interactions
│   │   └── role-overrides.yaml       # Role-level permission override tests
│   │                                  # Tests grant/deny mechanics on specific roles
│   └── servers/                       # Server permission validations
│       ├── manage-server.yaml        # Server management permission tests
│       ├── view-server.yaml          # Server viewing permission tests
│       ├── server-permissions.yaml   # Combined server permission tests
│       │                              # Tests permission hierarchy and interactions
│       ├── manage-nicknames.yaml     # Nickname management permission tests
│       │                              # Tests admin ability to edit any user's nickname
│       └── change-nickname.yaml      # Self-nickname change permission tests
│                                      # Tests user ability to change their own nickname
├── docker-compose.yml                # Docker setup for SpiceDB and zed CLI
└── README.md                         # Documentation for the authzed implementation

Schema Features:
- Permission overrides (grant/deny) similar to Discord
- Role-based permissions with server ownership
- Channel-level permission controls
- Role management capabilities (manage and view roles)
- Role-level permission overrides (grant/deny on specific roles)
- Server management capabilities (manage and view servers)
- Validation tests covering various scenarios
```
