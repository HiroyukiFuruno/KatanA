# WHY: fixtureやadvisory cacheのGit操作を、呼出元hookの別repo環境へ誤誘導しない。
release_git_environment_names="$(git rev-parse --local-env-vars)"
for release_git_environment_name in ${(f)release_git_environment_names}; do
    unset "$release_git_environment_name"
done
