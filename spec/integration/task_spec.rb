require 'support/integration'

RSpec.describe 'Managing tasks', type: :integration do
  include IntegrationSpec::Screenshots

  it 'displays tasks by status', js: true do
    visit '/'
    expect(page).to take_screenshot_of('tasks-by_statys')
  end
end
