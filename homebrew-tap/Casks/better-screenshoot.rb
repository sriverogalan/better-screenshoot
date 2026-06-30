cask "better-screenshoot" do
  version "0.2.1"

  on_arm do
    url "https://github.com/sriverogalan/better-screenshoot/releases/download/v#{version}/Better-Screenshoot_#{version}_aarch64.dmg"
    sha256 "PLACEHOLDER_ARM64_SHA256"
  end

  on_intel do
    url "https://github.com/sriverogalan/better-screenshoot/releases/download/v#{version}/Better-Screenshoot_#{version}_x64.dmg"
    sha256 "PLACEHOLDER_X64_SHA256"
  end

  name "Better Screenshoot"
  desc "Screenshot tool with a built-in editor"
  homepage "https://github.com/sriverogalan/better-screenshoot"

  no_quarantine true

  app "Better Screenshoot.app"
end
