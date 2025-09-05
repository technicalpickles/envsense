class Envsense < Formula
  desc "Cross-language library and CLI for detecting the runtime environment"
  homepage "https://github.com/technicalpickles/envsense"
  head "https://github.com/technicalpickles/envsense.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Test basic functionality - check that info command works
    output = shell_output("#{bin}/envsense info --json")
    assert_match "contexts", output
    assert_match "facets", output
    
    # Test check command with a basic predicate
    system bin/"envsense", "check", "--list"
  end
end
