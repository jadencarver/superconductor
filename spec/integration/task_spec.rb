require 'support/integration'
require 'faker'

RSpec.feature 'Tasks' do
  include IntegrationSpec::Screenshots

  STATUSES = ['Sprint', 'In Progress', 'In Review', 'Blocked', 'Done']
  SLEEP = 0.5

  scenario 'Setup', js: true do
    visit '/'
    page.execute_script("PM.open()");
    expect(panel).to have_text 'Setup Instructions'
    start_a_project
    create_lots_of_tasks
  end

  def start_a_project
    fill_in 'Project', with: 'Your Next Project'
    fill_in 'Description', with: 'Lorem Ipsum'
    fill_in 'message', with: 'Initial commit'
    expect(panel).to look_like('tasks-setup')
    click_button 'Save Update'
    expect(panel).to have_css('.tasks--backlog')
  end

  def create_a_new_task status
    puts "New Task: #{status}"
    click_button 'New Task'
    find('#__pm__commit__properties--status option', text: status).select_option
    fill_in 'Description', with: Faker::Hacker.say_something_smart
    sleep
  end

  def create_lots_of_tasks
    6.times do |i|
      create_a_new_task STATUSES[i % STATUSES.length]
    end
    create_a_new_task STATUSES[1]
    create_a_new_task STATUSES[0]
    create_a_new_task STATUSES[0]
    create_a_new_task STATUSES[-1]
    create_a_new_task STATUSES[-1]
    click_button 'Sprint'
    expect(panel).to have_css('.tiles')
    expect(panel).to look_like('tasks-by_status')
  end

  def panel
    page.find_by_id('__pm__panel')
  end

  def sleep(time = SLEEP)
    super(time)
  end

end
