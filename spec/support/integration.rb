require 'capybara/rspec'
require 'capybara/poltergeist'
require 'superconductor'
require 'yard'

module IntegrationSpec
  autoload :Screenshots, 'support/integration/screenshots'

  Capybara.javascript_driver = :poltergeist
  Capybara.app = YARD::Server::RackAdapter.new({
    'superconductor' => [YARD::Server::LibraryVersion.new('superconductor', Superconductor::VERSION, '.yardoc')]
  }, caching: true, single_library: true)
end
