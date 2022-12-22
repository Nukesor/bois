# Stages

These are the different stages that're done in a normal deployment job.

## Stage 0

- Compare the previous deployment with the actual files on disk.
- Notify the user if there're any differences between the deployed and temporary files.

## Stage 1

- Determine hostname
- Determine which files have precedence
- Determine all files that need to be copied

## Stage 2

- Check whether the files could theoretically be deployed.
- Copy the files to the temporary directory
- Save the state to disk.

## Stage 3

- Remove previously deployed files.
- Copy the files to the actual target locations.
