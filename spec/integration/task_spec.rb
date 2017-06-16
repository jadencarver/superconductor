require 'support/integration'

RSpec.feature 'Managing tasks' do
  include IntegrationSpec::Screenshots

  it 'displays tasks by status', js: true do
    visit '/docs/file/README.md'
    fill_in 'Project', with: 'Your Next Project'
    fill_in 'Description', with: 'Lorem Ipsum'
    expect(page.find('#__pm__panel')).to look_like('tasks-by_status')
    click_button 'New Task'
  end

end
