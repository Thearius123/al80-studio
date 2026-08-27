param()

$ErrorActionPreference = "Stop"

function Fail-D2A {
    param([string]$Reason)

    Write-Host ""
    Write-Host "============================================================"
    Write-Host "AL80 WINDOWS D2A PHYSICAL VALIDATION = FAIL"
    Write-Host "FAIL_REASON=$Reason"
    Write-Host "DEVICE_OPEN=NO"
    Write-Host "DEVICE_WRITE=NO"
    Write-Host "QMK_FLASH=NO"
    Write-Host "EEPROM_WRITE=NO"
    Write-Host "============================================================"
    exit 1
}

Write-Host "============================================================"
Write-Host "AL80 STUDIO — WINDOWS D2A PHYSICAL HID VALIDATION"
Write-Host "STRICTLY READ ONLY"
Write-Host "NO DEVICE OPEN / NO DEVICE WRITE"
Write-Host "============================================================"

if (-not $IsWindows) {
    Fail-D2A "not_windows"
}

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Enumerator = Join-Path $Root "al80-hid-enumerate.exe"

if (-not (Test-Path $Enumerator)) {
    Fail-D2A "al80_hid_enumerator_missing"
}

$RunningDaemon = Get-Process -Name "al80d" -ErrorAction SilentlyContinue

if ($null -ne $RunningDaemon) {
    Fail-D2A "al80d_is_running_stop_it_before_readonly_validation"
}

Write-Host ""
Write-Host "1. PACKAGE"
Write-Host "VALIDATOR_ROOT=$Root"
Write-Host "AL80_HID_ENUMERATOR=$Enumerator"
Write-Host "AL80D_RUNNING=NO"
Write-Host "PACKAGE_GATE=PASS"

Write-Host ""
Write-Host "2. READ-ONLY ENUMERATION"

$Output = @(& $Enumerator 2>&1)

foreach ($Line in $Output) {
    Write-Host $Line
}

$PassLine = $Output | Where-Object {
    $_ -eq "AL80_WINDOWS_HID_ENUMERATION=PASS"
}

$CountLine = $Output | Where-Object {
    $_ -match '^AL80_WINDOWS_HID_MATCH_COUNT=(\d+)$'
} | Select-Object -Last 1

$OpenLine = $Output | Where-Object {
    $_ -eq "AL80_WINDOWS_HID_OPEN=NO"
}

$WriteLine = $Output | Where-Object {
    $_ -eq "AL80_WINDOWS_HID_WRITE=NO"
}

if ($null -eq $PassLine) {
    Fail-D2A "enumerator_did_not_report_pass"
}

if ($null -eq $CountLine) {
    Fail-D2A "enumerator_match_count_missing"
}

if ($null -eq $OpenLine) {
    Fail-D2A "enumerator_open_invariant_missing"
}

if ($null -eq $WriteLine) {
    Fail-D2A "enumerator_write_invariant_missing"
}

$Match = [regex]::Match(
    [string]$CountLine,
    '^AL80_WINDOWS_HID_MATCH_COUNT=(\d+)$'
)

if (-not $Match.Success) {
    Fail-D2A "cannot_parse_match_count"
}

$Count = [int]$Match.Groups[1].Value

if ($Count -ne 1) {
    Fail-D2A "expected_exactly_one_AL80_raw_hid_match_found_$Count"
}

Write-Host ""
Write-Host "3. PHYSICAL READ-ONLY RESULT"
Write-Host "AL80_WINDOWS_HID_MATCH_COUNT=1"
Write-Host "AL80_WINDOWS_HID_EXACTLY_ONE=PASS"
Write-Host "DEVICE_OPEN=NO"
Write-Host "DEVICE_WRITE=NO"
Write-Host "PHYSICAL_WINDOWS_HID_ENUMERATION=PASS"

Write-Host ""
Write-Host "============================================================"
Write-Host "FINAL"
Write-Host "============================================================"
Write-Host "AL80_WINDOWS_STAGE_D2A_PHYSICAL_READONLY=PASS"
Write-Host "AL80_WINDOWS_HID_FILTER=VID_28E9_PID_30AF_USAGE_FF60_0061"
Write-Host "PHYSICAL_WINDOWS_HID_ENUMERATION=PASS"
Write-Host "PHYSICAL_WINDOWS_HID_OPEN=NO"
Write-Host "PHYSICAL_WINDOWS_HID_WRITE=NO"
Write-Host "PHYSICAL_WINDOWS_AUDIO=NOT_TESTED"
Write-Host "QMK_FLASH=NO"
Write-Host "EEPROM_WRITE=NO"
Write-Host "NEXT=WINDOWS_STAGE_D2B_CONTROLLED_RUNTIME_VALIDATION"
Write-Host "============================================================"
