require 'support/integration'

RSpec.feature 'Tasks' do
  include IntegrationSpec::Screenshots

  scenario 'Setup', js: true do
    visit '/'
    expect(panel).to have_text 'Setup Instructions'
    start_a_project
    create_a_new_task
  end

  def start_a_project
    fill_in 'Project', with: 'Your Next Project'
    fill_in 'Description', with: 'Lorem Ipsum'
    fill_in 'message', with: 'Initial commit'
    expect(panel).to look_like('tasks-setup')
    click_button 'Save Update'
    expect(panel).to have_css('.list--backlog')
  end

  def create_a_new_task
    click_button 'New Task'
    find('#__pm__commit__properties--status option', text: 'Sprint').select_option
    expect(panel).to look_like('tasks-backlog')
    click_button 'Sprint'
    expect(panel).to have_css('.tiles')
    expect(panel).to look_like('tasks-by_status')
  end

  def panel
    page.find_by_id('__pm__panel')
  end

end
