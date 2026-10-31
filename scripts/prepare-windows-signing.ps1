$ErrorActionPreference = "Stop"

$hasCertificate = -not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE)
$hasPassword = -not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_PASSWORD)

if ($hasCertificate -ne $hasPassword) {
  throw "Windows Authenticode signing requires both WINDOWS_CERTIFICATE and WINDOWS_CERTIFICATE_PASSWORD."
}

if (-not $hasCertificate) {
  "WINDOWS_SIGNING_ENABLED=false" | Out-File -FilePath $env:GITHUB_ENV -Append
  if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_OUTPUT)) {
    "tauri_release_args=" | Out-File -FilePath $env:GITHUB_OUTPUT -Append
  }
  Write-Host "Windows Authenticode signing is disabled."
  exit 0
}

$pfxPath = Join-Path $env:RUNNER_TEMP "lumadesk-signing.pfx"
$encodedCertificate = $env:WINDOWS_CERTIFICATE -replace '\s', ''
try {
  [System.IO.File]::WriteAllBytes(
    $pfxPath,
    [System.Convert]::FromBase64String($encodedCertificate)
  )

  $password = ConvertTo-SecureString $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
  $importArguments = @{
    FilePath = $pfxPath
    CertStoreLocation = "Cert:\CurrentUser\My"
    Password = $password
  }
  $importedCertificates = @(Import-PfxCertificate @importArguments)
  $signingCertificate = $importedCertificates |
    Where-Object HasPrivateKey |
    Select-Object -First 1
  if ($null -eq $signingCertificate) {
    throw "The imported PFX does not contain a signing certificate with a private key."
  }

  $config = @{
    bundle = @{
      windows = @{
        certificateThumbprint = $signingCertificate.Thumbprint
        digestAlgorithm = "sha256"
        timestampUrl = "http://timestamp.digicert.com"
      }
    }
  } | ConvertTo-Json -Depth 4 -Compress
  $configPath = Join-Path $PWD "src-tauri/tauri.release.json"
  Set-Content -LiteralPath $configPath -Value $config -Encoding utf8NoBOM

  "TAURI_CONFIG=$config" | Out-File -FilePath $env:GITHUB_ENV -Append
  "TAURI_RELEASE_ARGS=--config src-tauri/tauri.release.json" | Out-File -FilePath $env:GITHUB_ENV -Append
  "WINDOWS_SIGNING_ENABLED=true" | Out-File -FilePath $env:GITHUB_ENV -Append
  if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_OUTPUT)) {
    "tauri_release_args=--config src-tauri/tauri.release.json" |
      Out-File -FilePath $env:GITHUB_OUTPUT -Append
  }
  Write-Host "Windows Authenticode signing is enabled."
}
finally {
  Remove-Item -LiteralPath $pfxPath -Force -ErrorAction SilentlyContinue
}
