param(
  [Parameter(Mandatory = $true)]
  [string]$Target,
  [Parameter(Mandatory = $true)]
  [ValidateSet("x86_64", "aarch64")]
  [string]$Architecture,
  [Parameter(Mandatory = $true)]
  [string]$Version,
  [switch]$RequireAuthenticode
)

$ErrorActionPreference = "Stop"
$appPath = Join-Path "target/$Target/release" "lumadesk.exe"
$installerDirectory = "target/$Target/release/bundle/nsis"

if (-not (Test-Path -LiteralPath $appPath -PathType Leaf)) {
  throw "Missing packaged application executable: $appPath"
}

$installers = @(Get-ChildItem -LiteralPath $installerDirectory -Filter "*.exe" -File)
if ($installers.Count -ne 1) {
  throw "Expected exactly one NSIS installer in $installerDirectory, found $($installers.Count)."
}
$installer = $installers[0]
$expectedInstallerName = switch ($Architecture) {
  "x86_64" { "LumaDesk_${Version}_x64-setup.exe" }
  "aarch64" { "LumaDesk_${Version}_arm64-setup.exe" }
}
if ($installer.Name -ne $expectedInstallerName) {
  throw "Expected installer $expectedInstallerName, found $($installer.Name)."
}
$signaturePath = "$($installer.FullName).sig"
if (-not (Test-Path -LiteralPath $signaturePath -PathType Leaf)) {
  throw "Missing updater signature: $signaturePath"
}
if ((Get-Item -LiteralPath $signaturePath).Length -eq 0) {
  throw "Updater signature is empty: $signaturePath"
}

function Get-PeArchitecture([string]$Path) {
  $stream = [System.IO.File]::OpenRead((Resolve-Path -LiteralPath $Path))
  try {
    $reader = [System.IO.BinaryReader]::new($stream)
    $stream.Position = 0x3c
    $peOffset = $reader.ReadInt32()
    $stream.Position = $peOffset
    if ($reader.ReadUInt32() -ne 0x00004550) {
      throw "Invalid PE signature in $Path"
    }
    $machine = $reader.ReadUInt16()
    switch ($machine) {
      0x8664 { return "x86_64" }
      0xaa64 { return "aarch64" }
      default { throw "Unsupported PE machine 0x$($machine.ToString('x4')) in $Path" }
    }
  }
  finally {
    $stream.Dispose()
  }
}

$actualArchitecture = Get-PeArchitecture $appPath
if ($actualArchitecture -ne $Architecture) {
  throw "Expected $Architecture application executable, found $actualArchitecture."
}

$versionInfo = (Get-Item -LiteralPath $appPath).VersionInfo
if ($versionInfo.ProductName -ne "LumaDesk") {
  throw "Expected product name LumaDesk, found '$($versionInfo.ProductName)'."
}
if (-not $versionInfo.ProductVersion.StartsWith($Version, [System.StringComparison]::Ordinal)) {
  throw "Expected product version $Version, found '$($versionInfo.ProductVersion)'."
}

if ($RequireAuthenticode) {
  foreach ($path in @($appPath, $installer.FullName)) {
    $signature = Get-AuthenticodeSignature -LiteralPath $path
    if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
      throw "Authenticode verification failed for $path with status $($signature.Status)."
    }
  }
}

Write-Host "Verified Windows $Architecture package for LumaDesk $Version."
