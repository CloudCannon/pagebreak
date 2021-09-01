# frozen_string_literal: true

require "jekyll"

Jekyll::Hooks.register :site, :post_write do |site|
  Jekyll.logger.info "Jekyll Pagebreak:", "Running in-place on destination files"
  started = Time.now
  
  executable = File.expand_path("../../ext/pagebreak/pagebreak", __FILE__)
  system("#{executable} -s #{site.dest} -o #{site.dest}")

  rounded_time = ((Time.now - started) * 1000).round
  Jekyll.logger.info "Jekyll Pagebreak:", "Done in #{rounded_time} milliseconds"
end

