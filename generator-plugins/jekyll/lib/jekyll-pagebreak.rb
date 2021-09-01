# frozen_string_literal: true

require "jekyll"

Jekyll::Hooks.register :site, :post_write do |site|
  Jekyll.logger.info "Jekyll Pagebreak:", "Running in-place on destination files"
  started = Time.now
  
  executable = File.expand_path("../../pagebreak/pagebreak", __FILE__)
  system("#{executable} -s #{site.dest} -o #{site.dest}")

  Jekyll.logger.info "Jekyll Pagebreak:", "Done in #{(Time.now - started)*1000} milliseconds"
end

