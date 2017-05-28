require 'capybara/poltergeist'

module IntegrationSpec
  Capybara.javascript_driver = :poltergeist
  autoload :Screenshots, 'support/integration/screenshots'
end
