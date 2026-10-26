import os
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path(os.environ.get("WINDOWS_TRACE_SCRIPT_PATH", ROOT / "scripts/release/trace-windows-startup.ps1"))


class WindowsStartupTraceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = SCRIPT.read_text(encoding="utf-8")

    def test_requires_existing_candidate_and_diagnostic_directory(self):
        self.assertIn("[Parameter(Mandatory = $true)]", self.source)
        self.assertIn("[string]$ExecutablePath", self.source)
        self.assertIn("[string]$DiagnosticsDirectory", self.source)
        self.assertIn("Resolve-Path -LiteralPath $ExecutablePath", self.source)
        self.assertIn("Resolve-Path -LiteralPath $DiagnosticsDirectory", self.source)

    def test_wpr_is_fail_closed_and_only_general_profile_is_selected(self):
        self.assertIn("Get-Command -Name 'wpr.exe'", self.source)
        self.assertIn("Get-Command -Name 'tracerpt.exe'", self.source)
        self.assertIn("@('-profiles')", self.source)
        self.assertIn("@('-status')", self.source)
        self.assertIn("WPR is not recording\\.?", self.source)
        self.assertIn("Assert-NoActiveWprRecording -Text $beforeStatus.Text", self.source)
        self.assertIn("^\\s*WPR is not recording\\.?\\s*$", self.source)
        self.assertIn(r"$profiles.Text -notmatch '(?im)^\s*GeneralProfile", self.source)
        self.assertIn("@('-start', 'GeneralProfile', '-instancename', $wprInstance)", self.source)
        self.assertIn("-Arguments @('-status', '-instancename', $wprInstance)", self.source)
        self.assertIn("-Arguments @('-stop', $etl, 'bounded packaged startup diagnosis', '-skipPdbGen', '-instancename', $wprInstance)", self.source)
        self.assertIn("wpr-ownership.json", self.source)
        self.assertIn("start_attempted = $true", self.source)
        self.assertNotIn("-filemode", self.source.lower())
        for forbidden in ("-cancel", "-force", "-reset"):
            self.assertNotIn(forbidden, self.source.lower())
        start_index = self.source.index("@('-start', 'GeneralProfile', '-instancename', $wprInstance)")
        self.assertLess(self.source.index("Assert-NoActiveWprRecording -Text $beforeStatus.Text"), start_index)
        self.assertLess(self.source.index("$wprStartAttempted = $true"), start_index)
        self.assertIn("if ($wprStartAttempted)", self.source)
        self.assertNotIn("if ($wprOwned)", self.source)
        self.assertIn("wprFileVersion =", self.source)
        self.assertIn("@('-help', 'advanced')", self.source)


    def test_official_wpr_active_status_is_recognized_without_matching_idle_status(self):
        pattern = r"(?im)^\s*WPR recording is in progress\.{0,3}\s*$"
        self.assertIn(pattern, self.source)
        active = re.compile(pattern)
        self.assertRegex("WPR recording is in progress...", active)
        self.assertRegex("WPR recording is in progress", active)
        self.assertNotRegex("WPR is not recording", active)
        self.assertNotRegex("Status: Not Recording", active)
        self.assertIn("function Assert-NoActiveWprRecording", self.source)
        self.assertIn("WPR is not recording\\.?", self.source)

    def test_only_owned_process_is_stopped_after_fresh_identity_check(self):
        guard = self.source.index("$stillOwned = Assert-PackagedProcessIdentity")
        stop = self.source.index("$process.Kill()")
        self.assertLess(guard, stop)
        for identity in ("$actual.StartTime.ToUniversalTime().Ticks",
                         "Get-FileHash -LiteralPath $actualPath -Algorithm SHA256",
                         "Diagnostic process executable path changed"):
            self.assertIn(identity, self.source)
        self.assertIn("if ($ExpectedProcess.HasExited) { return $false }", self.source)

    def test_owned_collector_configuration_is_recorded_before_and_after_capture(self):
        arguments = "@('-status', 'collectors', '-details', '-instancename', $wprInstance)"
        self.assertEqual(self.source.count(arguments), 2)
        before = self.source.index("-Operation 'wpr-collectors-before'")
        launch = self.source.index("$process = Start-Process")
        cleanup = self.source.index("$process.Kill()")
        after = self.source.index("-Operation 'wpr-collectors-after'")
        stop = self.source.index("-Operation 'wpr-stop'")
        self.assertLess(before, launch)
        self.assertLess(cleanup, after)
        self.assertLess(after, stop)
        cleanup_boundary = self.source[after:stop]
        self.assertRegex(cleanup_boundary, r"\}\s*catch\s*\{")
        self.assertRegex(cleanup_boundary, r"\}\s*try\s*\{")
        self.assertIn("Collector status failed before capture", self.source)
        self.assertIn("Collector status failed after capture", self.source)
        self.assertIn("-OutputDirectory $DiagnosticsDirectory -TimeoutSeconds 30", self.source)

    def test_capture_is_bounded_and_uses_isolated_config_and_existing_diagnostics(self):
        self.assertIn("$process.WaitForExit(200)", self.source)
        self.assertIn("$timer.Elapsed.TotalSeconds -lt 10", self.source)
        self.assertIn("function Invoke-BoundedNative", self.source)
        self.assertIn("Start-Process -FilePath $Path -ArgumentList $argumentLine -PassThru -NoNewWindow", self.source)
        self.assertIn("$child.WaitForExit($TimeoutSeconds * 1000)", self.source)
        self.assertIn("$child.Kill()", self.source)
        self.assertIn("$child.WaitForExit(5000)", self.source)
        self.assertIn("$operationFailure = $_.Exception.Message", self.source)
        self.assertIn("else { $operationFailure +=", self.source)
        self.assertIn("Read-BoundedText", self.source)
        self.assertIn("1048576", self.source)
        self.assertIn("$exitCode = $child.ExitCode", self.source)
        self.assertNotIn("ReadToEnd", self.source)
        self.assertNotRegex(self.source, r"WaitForExit\s*\(\s*\)")
        for operation_deadline in ("-TimeoutSeconds 30", "-TimeoutSeconds 60"):
            self.assertIn(operation_deadline, self.source)
        self.assertIn('"empty-config-$nonce"', self.source)
        self.assertIn("KATANA_CONFIG_DIR", self.source)
        self.assertIn("KATANA_STARTUP_HEARTBEAT", self.source)
        self.assertIn("$env:DEBUG = 'true'", self.source)
        self.assertIn("-RedirectStandardOutput $stdout -RedirectStandardError $stderr", self.source)
        self.assertIn("diagnostic-stdout.log", self.source)
        self.assertIn("diagnostic-stderr.log", self.source)


    def test_workflow_runs_diagnostic_only_after_original_smoke_failure(self):
        workflow = (ROOT / ".github/workflows/build-and-release.yml").read_text(encoding="utf-8")
        smoke = workflow.index("      - name: Smoke launch packaged Windows app")
        diagnostic = workflow.index("      - name: Trace failed packaged Windows startup")
        upload = workflow.index("      - name: Upload Windows startup diagnostics")
        self.assertLess(smoke, diagnostic)
        self.assertLess(diagnostic, upload)
        diagnostic_block = workflow[diagnostic:upload]
        self.assertIn("if: failure() && steps.windows_startup_smoke.outcome == 'failure'", diagnostic_block)
        self.assertIn("scripts/release/trace-windows-startup.ps1", diagnostic_block)
        self.assertIn("-ExecutablePath", diagnostic_block)
        self.assertIn("-DiagnosticsDirectory", diagnostic_block)
        smoke_block = workflow[smoke:diagnostic]
        for retained_condition in (
            "Start-Sleep -Milliseconds 200",
            "foreach ($attempt in 1..10)",
            "if ($peakRss -gt 512MB)",
            "if ($fontBytes -gt 134217728)",
            "Assert-PackagedProcessIdentity",
            "Packaged app did not reach first frame",
        ):
            self.assertIn(retained_condition, smoke_block)
        self.assertNotIn("continue-on-error", smoke_block)

    def test_results_are_observational_and_tracerpt_is_optional(self):
        self.assertIn("$null -ne $tracerptCommand", self.source)
        self.assertIn("-Operation 'tracerpt-conversion' -OutputDirectory $DiagnosticsDirectory -TimeoutSeconds 60", self.source)
        self.assertIn("windows-startup-diagnostic.csv", self.source)
        self.assertIn("Diagnostic observation only; this result does not pass or replace the original smoke.", self.source)
        self.assertIn("original smoke result is unchanged", self.source)
        self.assertNotIn("Write-Host 'PASS", self.source)


if __name__ == "__main__":
    unittest.main()
