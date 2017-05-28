require 'support/integration'

RSpec.feature 'Managing tasks' do
  include IntegrationSpec::Screenshots

  it 'displays tasks by status', js: true do
    visit 'http://www.google.com'
    expect(page).to take_screenshot_of('tasks-by_status')
  end
end
