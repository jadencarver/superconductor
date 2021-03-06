$:.unshift File.join(File.dirname(__FILE__), 'lib')
require 'superconductor'
require 'yard'

use Superconductor::Middleware
#use Rack::Static, root: "doc", urls: ["/css", "/js"], index: 'index.html'
#use Rack::Static, root: "target", urls: ["/doc"], index: 'index.html'
run YARD::Server::RackAdapter.new({
  'superconductor' => [YARD::Server::LibraryVersion.new('superconductor', Superconductor::VERSION, '.yardoc')]
}, caching: true, single_library: true)
