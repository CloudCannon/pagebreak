$LOAD_PATH.unshift File.expand_path("../lib", __dir__)
require "jekyll-pagebreak/version"

# Make ruby think it's building a native extension
dummy_make_content = "make:\n" \
                     "\t:\n" \
                     "install:\n" \
                     "\t:\n" \
                     "clean:\n" \
                     "\t:\n"
File.write('Makefile', dummy_make_content)

# Bail out if the binary already exists
if File.file?("pagebreak/pagebreak")
    puts "Pagebreak binary already exists"
    exit 0
end

puts "Installing Pagebreak"

# Construct our release url
def build_release_url(arch, version)
  "https://github.com/CloudCannon/pagebreak/releases/download/v#{version}/pagebreak-v#{version}-#{arch}.tar.gz"
end

release_url = if RUBY_PLATFORM =~ /x86_64-darwin/
    build_release_url("x86_64-apple-darwin", JekyllPagebreak::VERSION)
elsif RUBY_PLATFORM =~ /x86_64-linux/
    build_release_url("x86_64-unknown-linux-musl", JekyllPagebreak::VERSION)
else
    puts "Unsupported platform: #{RUBY_PLATFORM}\nPlease open an issue at https://github.com/CloudCannon/pagebreak/issues"
    exit 1
end

# Download the tar.gz from GitHub
require 'open-uri'
if open(release_url)
  puts "Downloading Pagebreak v#{JekyllPagebreak::VERSION} from GitHub"
  open('pagebreak.tar.gz', 'wb') do |file|
    file << open(release_url).read
  end
else
  puts "Could not find Pagebreak v#{JekyllPagebreak::VERSION}"
  exit 1
end

# Extract the tar.gz using the gem helper
require 'rubygems/package'
Dir.mkdir('pagebreak') unless Dir.exist?('pagebreak')
Gem::Package.new("").extract_tar_gz(File.open("pagebreak.tar.gz", "rb"), "pagebreak")

# Clean up after ourselves
FileUtils.rm('pagebreak.tar.gz')

puts "Pagebreak Installed!"
