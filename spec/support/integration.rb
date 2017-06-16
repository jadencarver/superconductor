require 'capybara/rspec'
#require 'capybara/poltergeist'
require 'superconductor'
require 'yard'

ENV['GIT_DIR'] = ENV['PWD'] + '/tmp/dummy'

module IntegrationSpec
  autoload :Screenshots, 'support/integration/screenshots'

  #Capybara.javascript_driver = :poltergeist

  Capybara.app = Rack::Builder.app do
    use Superconductor::Middleware
    run YARD::Server::RackAdapter.new({
      'superconductor' => [YARD::Server::LibraryVersion.new('superconductor', Superconductor::VERSION, '.yardoc')]
    }, caching: true, single_library: true)
  end

end
