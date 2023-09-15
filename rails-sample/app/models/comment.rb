module Sample
  module A::B
    CONSTANT = 'constant'.freeze

    class Comment < ApplicationRecord
      belongs_to :post, class_name: 'Post'
    end
  end
end
