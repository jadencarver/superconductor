$:.unshift File.join(File.dirname(__FILE__), 'lib')
require 'superconductor'

use Superconductor::Middleware
run Superconductor::Documentation.new(
  path: File.join(File.dirname(__FILE__), 'doc'),
  index: 'introduction.html'
)
