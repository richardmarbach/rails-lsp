class Comment < ApplicationRecord
  belongs_to :post, class_name: "Post"
end
