cask "better-screenshoot" do
  version "0.3.1"

  on_arm do
    url "https://github.com/sriverogalan/better-screenshoot/releases/download/v#{version}/Better-Screenshoot_#{version}_aarch64.dmg"
    sha256 "7cffbbfad2fd7e41327639526fe5d77cc561cc67648fa21b9a2545bcb9aa5570"
  end

  on_intel do
    url "https://github.com/sriverogalan/better-screenshoot/releases/download/v#{version}/Better-Screenshoot_#{version}_x64.dmg"
    sha256 "722105dcbb5578a2f02166906b7842c1673d503586569e655d6512040b835b71"
  end

  name "Better Screenshoot"
  desc "Screenshot tool with a built-in editor"
  homepage "https://github.com/sriverogalan/better-screenshoot"

  app "Better Screenshoot.app"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{appdir}/Better Screenshoot.app"]
  end
end
