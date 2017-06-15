require 'support/integration'

RSpec.feature 'Managing tasks' do
  include IntegrationSpec::Screenshots

  it 'displays tasks by status', js: true do
    visit '/docs/file/README.md'
    #page.evaluate_script("$('#__pm')[0].getBoundingClientRect();")
    panel = page.find('#__pm__panel')
    expect(panel).to look_like('tasks-by_status')
  end

end
