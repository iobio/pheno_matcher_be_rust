### Intro

---

This is a backend service for the pheno-matcher project. The server and hpo functions are implmented in rust.

The docker container can be found here: `emersonlebleu/pheno_matcher_be_rust-server`

Currently there is a `amd64` tagged version that is compatable with our CHPC environment.

### Deploying General

## Build With Docker Locally

`docker build --platform linux/amd64 -t emersonlebleu/pheno_matcher_be_rust-server:amd64v2.1 .`

## Push the container to docker hub THEN --> Pull the Container

`singularity pull --name pheno_matcher_be_v2.1.sif docker://emersonlebleu/pheno_matcher_be_rust-server:amd64v2.1`

## Then Run

`singularity run --bind /ssd/emerson/pheno_matcher_be_rust/data/:/data pheno_matcher_be_v2.1.sif`

## Details

---

The docker container will not run as is out of the box for data security reasons. The rust `main` function will expect an external `/data` volume to be mounted to the root of your container. This should contain a csv of your patient information in the appropriate format for the application to use.

Other non-sensitive information has been copied to the docker container itself.

### CHPC Environment Instructions

---

For iobio.team purposes this service is hosted in our protected environment and runs in a singularity container.

Instruction on how to use a docker container in our internal team's environment can be found here: [CHPC Container Documenation](https://www.chpc.utah.edu/documentation/software/singularity.php#module)

To run the service you will need to mount or `--bind` the host folder that contains your data to the `/data` folder when running your container.

For example the following command:

```
singularity run --bind /username/datafolder:/data docker://emeronlebleu/pheno_matcher_be_rust-server:amd64
```

Here we are using a container from the docker hub and running it with singularity. Before we do that however we use the `--bind` flag and specify the originFolder:containerFolder that should contain our main data.

**NOTE:** the server will expect this containerFolder to be `data` at the root of your container as shown in the example.
**NOTE:** each new version needs to be built, tagged, and added to the docker hub. On CHPC stop the ongoing singularity container and run again with the new docker container.

**UPDATE** If there is already .sif built you can run something like

```
singularity run --bind /ssd/emerson/pheno_matcher_be_rust/data/:/data pheno_matcher_be_1.3.sif
```
