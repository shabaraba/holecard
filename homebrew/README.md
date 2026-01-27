# Homebrew Formula

## Setup

1. Use existing tap repository:
```bash
git clone https://github.com/shabarba/homebrew-tap
cd homebrew-tap
```

2. After each release, update the formula:
```bash
# Download the release archives and calculate SHA256
VERSION=0.1.0
curl -LO https://github.com/shabarba/holecard/releases/download/v${VERSION}/hc-aarch64-apple-darwin.tar.gz
curl -LO https://github.com/shabarba/holecard/releases/download/v${VERSION}/hc-x86_64-unknown-linux-gnu.tar.gz

# Calculate SHA256
shasum -a 256 hc-aarch64-apple-darwin.tar.gz
shasum -a 256 hc-x86_64-unknown-linux-gnu.tar.gz

# Update hc.rb.template with version and SHA256 values
# Then copy to your tap repository
cp hc.rb homebrew-tap/Formula/

# Commit and push
cd homebrew-tap
git add Formula/hc.rb
git commit -m "feat: update hc to v${VERSION}"
git push
```

3. Users can install with:
```bash
brew tap shabarba/tap
brew install hc
```

## Automation (optional)

You can automate formula updates using GitHub Actions in your tap repository.
