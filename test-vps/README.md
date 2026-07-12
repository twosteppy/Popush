# Popush integration test VPS (§23.3)

A throwaway container running `sshd` plus a sample site for each service type
(docker, systemd, pm2, static). The integration suite connects to it over SSH and
runs real operations — the test that catches the bugs that matter: escaping,
status parsing, and the Ship It pipeline end to end.

It never touches your real environment (Agent Rule 6). The key pair it uses is
generated fresh into this directory and gitignored; the container is disposable.

## Bring it up

```sh
./generate-key.sh                 # writes ./test_key and ./authorized_keys
docker compose up -d --build
ssh -i ./test_key -p 2222 deploy@localhost   # sanity check
```

## Run the integration tests

The Popush SSH layer authenticates through `ssh-agent`, so load the throwaway key
into an agent and hand the tests the container's host key:

```sh
eval "$(ssh-agent -s)"
ssh-add ./test_key

export POPUSH_TEST_VPS=localhost:2222
export POPUSH_TEST_KNOWN_HOSTS="$(ssh-keyscan -p 2222 localhost 2>/dev/null | sed 's/^\[localhost\]:2222/localhost/')"

cargo test -p popush --test integration -- --ignored --nocapture
```

`.github/workflows/ci.yml` performs exactly these steps in the `integration` job.

## Tear it down

```sh
docker compose down -v
rm -f test_key test_key.pub
```
