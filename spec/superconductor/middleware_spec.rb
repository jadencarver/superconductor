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

  describe 'javascript' do
    let(:path) { "/__pm.js" }

    it 'is served' do
      status, headers, body = subject.call(env)
      expect(status).to be 200
      expect(headers).to be_a Hash
      expect(headers['Content-Type']).to eq "text/javascript"
      expect(body.join).to include("PM.superconductor");
    end

  end

end
