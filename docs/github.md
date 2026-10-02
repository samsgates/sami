# Prepare a GitHub repository

Unzip into a new folder and review `README.md`, `SECURITY.md`, `LICENSE`, `NOTICE`, the PRD coverage and verification report. The distribution excludes live credentials, customer data, dependencies and compiled platform binaries. Commit lockfiles; never commit generated `.env`, development key files, database state, downloaded private exports or customer sources. Verify `.gitignore` against your deployment paths before staging.

Run `sh deploy/scripts/check.sh`, and run the explicit PostgreSQL integration test against a new isolated database. GitHub Actions defines quality, dependency-audit, image-build/SBOM and vulnerability gates. A local passing test run is not a claim that those hosted jobs have run. Configure security reporting, branch protection, required jobs and a maintainer before publishing a production release.

From the unpacked root initialize your own repository with `git init -b main`. Inspect `git status --short`, stage the intended source, then create your reviewed initial commit. Create the intended GitHub repository using your account, add its actual remote and push your chosen branch. Repository visibility, owner, URL and release signing must be chosen by the maintainer; no repository has been created or published as part of this ZIP delivery.

SHA-256 file hashes in `MANIFEST.sha256` validate this source distribution. Generated manifests are inventory/integrity aids, not an externally authenticated release signature. For a public binary/image release, pin source and image digests, run container scans, generate SBOMs, sign artifacts using your reviewed release infrastructure and retain the evidence.
