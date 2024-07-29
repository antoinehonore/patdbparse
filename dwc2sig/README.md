# Getting started

- Make sure that apptainer/singularity is installed.
You can find instructions there: ![https://apptainer.org/docs/admin/main/installation.html#installation-on-linux](https://apptainer.org/docs/admin/main/installation.html#installation-on-linux)


- Build the container
```bash
cd dwc2sig/singularity
make build
```

- Test 
```bash
cd ../dwc2sig
singularity exec ../singularity/env.dir/ /opt/dwc2sig/target/release/patdb_bin --verbose summarize -i examples/data_monitor/pat20 -m LF -o examples/example_output
```

## GPG
- Make sure that the data encryption gpg key is imported on your host
```bash
gpg -k
```

- Make sure that preseting a passphrase in the agent is allowed, if not:
```bash
echo "allow-preset-passphrase" >> ~/.gnupg/gpg-agent.conf
gpg-connect-agent reloadagent /bye
```