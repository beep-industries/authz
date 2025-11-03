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



