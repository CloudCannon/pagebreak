# frozen_string_literal: true

require "jekyll"

Jekyll::Hooks.register :site, :post_write do |site|
  puts "Running pagebreak in-place"
  executable = File.expand_path("../../exe/pagebreak", __FILE__)
  system("#{executable} -s #{site.dest} -o #{site.dest}")
end

