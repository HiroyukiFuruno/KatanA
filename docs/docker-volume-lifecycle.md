# Docker verification volume lifecycle

New KatanA volumes carry repository-owner and purpose labels. Existing unlabelled volumes and resources owned by other repositories are preserved.

## Preview and targeted cleanup

```sh
rtk proxy just docker-volumes-preview
rtk proxy just docker-volume-delete-ui-run docker-ui-functional.RUN_ID
rtk proxy just docker-volume-delete-cache VOLUME_NAME
```

Preview never deletes data. Cleanup selects one run or one cache by name. Cache deletion also requires typing the exact name. There is no global prune or force option.

## Persistent caches

Linux/Windows target volumes are reusable fixed caches. They are retained between checks and can be removed explicitly only when no container references them, including created and stopped containers. Preview shows Docker-reported size and referenced container IDs/states when the engine provides them; unsupported usage data is reported as `unknown`. Cargo registry and xwin volumes are listed as shared caches with the same size/reference visibility, but are never accepted by the cache deletion command.

## Per-run verification evidence

Successful runs keep the existing cleanup behavior after normal application exit and evidence export. Failed runs retain their resources for investigation.

An export receipt is written only after the application has stopped and the complete export succeeds. It binds the owner, run, exact X11/evidence pair and host evidence-tree SHA-256. Later cleanup requires matching labels, both volumes, no container references, an unchanged verified export and 30 elapsed days. Thirty days is the conservative manual-retention default, not a scheduled automatic deletion or a release approval deadline. Expired resources remain until explicitly selected.

Missing/modified exports, incomplete pairs, unknown ownership and historical unlabelled volumes remain protected.

## Verification

```sh
rtk proxy just test-docker-volume-lifecycle
rtk proxy just check-docker-volume-lifecycle
```

The latter uses disposable real-Docker fixtures to verify target and shared cache visibility, cache deletion guards and revalidation, created/running/stopped references, missing exports, expired verified exports, reruns, and the producer's normal, forced-stop and export-failure cleanup paths. It also verifies that the pre-existing volume-name inventory is unchanged. These checks do not replace application Linux/Windows checks or functional UI acceptance.
