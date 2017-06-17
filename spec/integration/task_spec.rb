require 'support/integration'

RSpec.feature 'Tasks' do
  include IntegrationSpec::Screenshots

  let(:panel) { page.find('#__pm__panel') }

  scenario 'Setup', js: true do
    visit '/'
    expect(panel).to have_text 'Setup Instructions'
    within panel do
      start_a_project
      create_a_new_task
    end
  end

  def start_a_project

    fill_in 'Project', with: 'Your Next Project'
    fill_in 'Description', with: 'Lorem Ipsum'
    fill_in 'message', with: 'Initial commit'
    expect(panel).to look_like('tasks-setup')

    panel.click
    sleep 1

    click_button 'Save Update'

    expect(panel).to have_css('.list-backlog')
  end

  def create_a_new_task
    click_button 'New Task'
    find('#__pm__commit__properties--status option', text: 'Sprint').select_option
    expect(page.find('#__pm__panel')).to look_like('tasks-backlog')
  end

end
