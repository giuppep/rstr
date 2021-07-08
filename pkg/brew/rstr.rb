class RstrBin < Formula
  version "v0.1.0"
  desc "A simple content addressable blob store with a web interface."
  homepage "https://github.com/giuppep/rstr"

  if OS.mac?
      url "https://github.com/giuppep/rstr/releases/download/#{version}/rstr-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "ccae4b910bccd82792b7fb2eca5e8f68591952861dad9458da81ced75d89deef"
  end

  def install
    bin.install "rstr"
  end
end