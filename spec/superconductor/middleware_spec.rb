require 'spec_helper'
require 'open-uri'
require 'superconductor/middleware'

RSpec.describe Superconductor::Middleware do

  subject { described_class.new env }

  let(:env) {
    {
      "PATH_INFO" => path
    }
  }

  it 'serves javascript' do
    env["PATH_INFO"] = "/__pm.js"
    status, headers, body = subject.call(env)
    expect(status).to be 200
    expect(headers).to be_a Hash
    expect(headers['Content-Type']).to eq "text/javascript"
    expect(body).to include("PM.superconductor");
  end

end
