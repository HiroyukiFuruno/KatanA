## ADDED Requirements

### Requirement: Update discovery must survive public API rate limits

KatanA MUST continue update discovery through GitHub's public latest-release redirect when the unauthenticated REST API is rate limited. The fallback MUST derive a valid SemVer tag from the final release URL without requiring user credentials.

#### Scenario: Shared-IP API quota is exhausted

- **WHEN** the latest-release REST API returns status 403 or 429
- **THEN** KatanA follows the public `/releases/latest` redirect
- **THEN** the final `/releases/tag/{version}` URL supplies the SemVer tag and release page URL
- **THEN** a successful fallback does not surface the API status as an update-check error
- **THEN** an unusable proxy does not replace the already received rate-limit status before fallback
