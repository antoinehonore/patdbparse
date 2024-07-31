# Getting started

- Make sure that apptainer/singularity is installed on your host.
You can find instructions there: ![https://apptainer.org/docs/admin/main/installation.html#installation-on-linux](https://apptainer.org/docs/admin/main/installation.html#installation-on-linux)


- Build the container (You need root access to the host)
```bash
cd singularity
make build
make pkg
```

- Make sure that the data encryption gpg key is imported on your host
```bash
gpg -k
```

- Make sure that preseting a passphrase in the agent is allowed, if not:
```bash
echo "allow-preset-passphrase" >> ~/.gnupg/gpg-agent.conf
gpg-connect-agent reloadagent /bye
```

- Test 
```bash
cd dwc2sig
singularity exec ../../singularity/env.dir/ /opt/dwc2sig/target/release/patdb_bin --verbose summarize -i examples/data_monitor/pat20 -m LF -o examples/example_output
```

## Notes
1. If you see an error like this:
```bash
thread 'main' panicked at src/main.rs:270:49:
index out of bounds: the len is 0 but the index is 19
```
It is likely because the patient map file was not decrypted. I haven't been able to find a good solution for `gpg` to work correctly inside the container with a private key requiring a passphrase.

Work arounds:
- do not use the container and build the binary on your host directly (See the command in `singularity/thecontainer.def`)
- (unsafe) decrypt the map file, and place it in the same folder as the encrypted one with filename (`PatientsMapping.txt`)

To investigate this further: the following command should run without prompting for a passphrase:
```bash
singularity exec ../singularity/env.dir/ gpg -d --batch --yes --passphrase-file /opt/psql/gpg_antoine_pfile.txt examples/data_monitor/PatientsMapping.txt.gpg
```

2. Make sure the map files are utf-8 encoded. They originated from a windows system and the map files from early datasets might be utf16. to fix this:
```bash
file -i PatientsMapping.txt
test.csv: text/plain; charset=utf-8
```
You can use the `iconv` tool to convert to `utf-8` if it's not already the case.

3. There are scripts to run the binaries on multiple `pat[id]` folders sequentially.
Make sure that the data and the binary folder path are configured properly in either the Makefile: `dwc2sig/scripts/update.mk`; or the shell script: `dwc2sig/scripts/compute-summaries.sh`.